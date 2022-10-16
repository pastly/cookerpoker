use std::error::Error;
use std::io::{stdin, stdout, BufRead, Write};

use poker_core::bet::{BetAction, BetStatus};
use poker_core::deck::DeckSeed;
use poker_core::state::{GameState, State};
use poker_core::{Currency, GameError};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, default_value = "6")]
    n_players: u8,
    #[structopt(long, default_value = "100000")]
    start_stack: Currency,
    #[structopt(long, default_value)]
    seed: DeckSeed,
    #[structopt(long, help = "Silence game prompts (useful for tests with set input)")]
    no_prompts: bool,
    #[structopt(
        long,
        help = "Silence post-game info dump (useful when not doing tests)"
    )]
    no_summary: bool,
    #[structopt(long, help = "Keep playing new hands until quit command is given")]
    multi_round: bool,
}

#[derive(Debug, Copy, Clone)]
enum Command {
    BetAction(BetAction),
    Info,
    Quit,
    Help,
}

fn print_help() {
    println!("Known commands are:");
    for (cmds, desc) in [
        ("(h)elp", "This output."),
        ("(i)nfo", "Get info on the current state of the hand."),
        ("(q)uit", "Stop playing."),
        ("(ch)eck", "Current player checks."),
        ("(f)old", "Current player folds."),
        ("(c)all X", "Current player calls for X."),
        ("(b)et X", "Current player makes a bet of X."),
        ("(r)aise X", "Current player raises to X."),
        ("(a)llin X", "Currenet player goes all in for X."),
    ] {
        println!("  {:9}: {}", cmds, desc);
    }
    println!("All bet amounts are in pennies and are a player's total wager for the");
    println!("current betting round.");
}

fn try_parse_bet_action(buf: &str) -> Result<BetAction, Box<dyn Error>> {
    let words: Vec<&str> = buf.split_whitespace().collect();
    if words.is_empty() {
        return Err("Empty input".into());
    } else if words.len() != 1 && words.len() != 2 {
        return Err("Wrong number of words".into());
    }
    let ba = match words[0] {
        "fold" | "f" => BetAction::Fold,
        "check" | "ch" => BetAction::Check,
        "call" | "c" | "bet" | "b" | "raise" | "r" | "allin" | "all" | "a" => {
            if words.len() != 2 {
                return Err("No second word".into());
            } else if words[1].is_empty() {
                return Err("Empty second word".into());
            }
            let amt: Currency = words[1].parse::<i32>()?;
            match words[0].chars().next().unwrap() {
                'c' => BetAction::Call(amt),
                'b' => BetAction::Bet(amt),
                'r' => BetAction::Raise(amt),
                'a' => BetAction::AllIn(amt),
                _ => unreachable!(),
            }
        }
        _ => return Err("Unable to parse first word as bet action".into()),
    };
    Ok(ba)
}

fn try_parse_command(stream: &mut dyn BufRead) -> Result<Command, Box<dyn Error>> {
    let mut s = String::new();
    let n = stream.read_line(&mut s)?;
    let words: Vec<&str> = s.split_whitespace().collect();
    if n == 0 {
        return Ok(Command::Quit);
    } else if s.starts_with('#') {
        return Err("Comment line".into());
    } else if words.is_empty() {
        return Err("Empty input".into());
    } else if let Ok(ba) = try_parse_bet_action(&s) {
        return Ok(Command::BetAction(ba));
    } else if words.len() != 1 {
        return Err("Wrong number of words".into());
    }
    let c = match words[0] {
        "info" | "i" => Command::Info,
        "quit" | "q" => Command::Quit,
        "help" | "h" => Command::Help,
        _ => return Err("Unable to parse first word as a command".into()),
    };
    Ok(c)
}

fn prompt(q: &str, display_prompts: bool) -> Result<Command, Box<dyn Error>> {
    if display_prompts {
        println!("{}", q);
    }
    let c = loop {
        if display_prompts {
            print!("> ");
            stdout().flush()?;
        }
        match try_parse_command(&mut stdin().lock()) {
            Ok(c) => break c,
            Err(e) => println!("{}", e),
        }
    };
    Ok(c)
}

fn print_player_info(state: &GameState, prefix: &str) {
    for (idx, player) in state.players.players_iter_with_index() {
        let mut tokens = vec![];
        if idx == state.players.token_dealer {
            tokens.push("D");
        }
        if idx == state.players.token_sb {
            tokens.push("SB");
        }
        if idx == state.players.token_bb {
            tokens.push("BB");
        }
        println!(
            "{}{:>4} Player {:>2} [{:>8}] {:<9} {}",
            prefix,
            tokens.join("/"),
            player.id,
            player.stack,
            match player.bet_status {
                BetStatus::Folded => "Folded".to_string(),
                BetStatus::Waiting => "Waiting".to_string(),
                BetStatus::In(x) => x.to_string(),
                BetStatus::AllIn(x) => format!("{} (all in)", x),
            },
            match player.pocket {
                None => String::new(),
                Some(p) => p[0].to_string() + &p[1].to_string(),
            }
        );
    }
    println!("Pot total value: {}", state.pot_total_value());
}

/// Run a single hand.
///
/// On clean exit, returns true if the user gave the quit command or for any other reason we should
/// not run another game even if the user gave the --multi-round flag. Otherwise false.
fn single_hand(
    state: &mut GameState,
    seed: DeckSeed,
    display_prompts: bool,
) -> Result<bool, Box<dyn Error>> {
    match state.start_hand_with_seed(seed) {
        Ok(_) => {}
        Err(e) => match e {
            GameError::NotEnoughPlayers => return Ok(true),
            _ => return Err(e.into()),
        },
    };
    if display_prompts {
        //println!("--- Begin hand {:2} ---", gip.hand_num);
        println!("DeckSeed: {}", seed);
        print_player_info(state, "  ");
    }
    loop {
        if matches!(state.state(), State::EndOfHand) {
            return Ok(false);
        }
        let (_, player) = state.nta().unwrap();
        let pocket = player.pocket.unwrap();
        let q = format!(
            "Community: {}\nPlayer {}'s action? {} {}",
            state
                .community
                .iter()
                .take_while(|c| c.is_some())
                .map(|c| c.unwrap().to_string())
                .collect::<Vec<_>>()
                .join(""),
            player.id,
            pocket[0],
            pocket[1]
        );
        match prompt(&q, display_prompts)? {
            Command::Info => {
                if display_prompts {
                    print_player_info(state, "  ");
                }
            }
            Command::Quit => return Ok(true),
            Command::Help => {
                if display_prompts {
                    print_help();
                }
            }
            Command::BetAction(ba) => match state.player_action(player.id, ba) {
                Ok(_) => { /*println!("p{} did {}", player.id, ba);*/ }
                Err(e) => println!("{}", e),
            },
        }
        if display_prompts {
            println!();
        }
    }
}

fn print_test_info(state: &GameState) -> Result<(), Box<dyn Error>> {
    println!("state {:?}", state.state());
    println!("current_bet {}", state.current_bet());
    println!("min_raise {}", state.min_raise());
    println!("pot.total_value {}", state.pot_total_value());
    println!(
        "community {} {} {} {} {}",
        match state.community[0] {
            None => "None".to_string(),
            Some(c) => c.to_string(),
        },
        match state.community[1] {
            None => "None".to_string(),
            Some(c) => c.to_string(),
        },
        match state.community[2] {
            None => "None".to_string(),
            Some(c) => c.to_string(),
        },
        match state.community[3] {
            None => "None".to_string(),
            Some(c) => c.to_string(),
        },
        match state.community[4] {
            None => "None".to_string(),
            Some(c) => c.to_string(),
        },
    );
    for player in state.players.players_iter() {
        println!("player {} bank {}", player.id, player.stack);
        println!("player {} bet_status {}", player.id, player.bet_status);
        println!(
            "player {} pocket {}",
            player.id,
            match player.pocket {
                None => "None".to_string(),
                Some(pocket) => format!("{}{}", pocket[0], pocket[1]),
            }
        );
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let mut state = GameState::default();
    for n in 1..opt.n_players + 1 {
        state.try_sit(n.into(), opt.start_stack)?;
    }
    if !opt.no_prompts {
        println!(
            "{} players seated with {} each",
            opt.n_players, opt.start_stack
        );
    }
    loop {
        let wants_quit = single_hand(&mut state, opt.seed, !opt.no_prompts)?;
        if wants_quit || !opt.multi_round {
            break;
        }
    }
    if !opt.no_summary {
        print_test_info(&state)?;
    }
    Ok(())
}
