use std::error::Error;
use std::io::{stdin, stdout, BufRead, Write};
use std::num::ParseIntError;

use poker_core::{
    deck::DeckSeed,
    game::{players::BetStatus, BetAction, Currency},
    PlayerId,
};
use poker_core::game::table::{GameState, GameInProgress};
use structopt::StructOpt;

fn parse_currency(src: &str) -> Result<Currency, ParseIntError> {
    Ok(src.parse::<i32>()?.into())
}

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, default_value = "6")]
    n_players: u8,
    #[structopt(long, default_value="100000", parse(try_from_str=parse_currency))]
    start_stack: Currency,
    #[structopt(long, default_value)]
    seed: DeckSeed,
    #[structopt(
        long,
        help = "Silence game prompts (useful for tests with set input)"
    )]
    no_prompts: bool,
    #[structopt(
        long,
        help = "Silence post-game info dump (useful when not doing tests)"
    )]
    no_summary: bool,
    #[structopt(
        long,
        help = "Keep playing new hands until quit command is given"
    )]
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
            let amt: Currency = words[1].parse::<i32>()?.into();
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
    } else if s.chars().next().unwrap() == '#' {
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

fn print_player_info(gip: &GameInProgress, players: &[PlayerId], prefix: &str) {
    for player in players {
        let info = gip.get_player_info(*player).expect("Player must exist");
        let mut tokens = vec![];
        if info.is_dealer {
            tokens.push("D");
        }
        if info.is_small_blind {
            tokens.push("SB");
        }
        if info.is_big_blind {
            tokens.push("BB");
        }
        println!(
            "{}{:>4} Player {:>2} [{:>8}] {:<9} {}",
            prefix,
            tokens.join("/"),
            info.id,
            format!("{}", info.monies),
            match info.bet_status {
                BetStatus::Folded => "Folded".to_string(),
                BetStatus::Waiting => "Waiting".to_string(),
                BetStatus::In(x) => x.to_string(),
                BetStatus::AllIn(x) => format!("{} (all in)", x),
            },
            match info.pocket {
                None => String::new(),
                Some(p) => p[0].to_string() + &p[1].to_string(),
            }
        );
    }
    println!("Pot total value: {}", gip.pot.total_value());
}

/// Run a single hand.
///
/// On clean exit, returns true if the player gave the quit command, otherwise false.
fn single_hand(
    gip: &mut GameInProgress,
    players: &[PlayerId],
    seed: &DeckSeed,
    display_prompts: bool,
) -> Result<bool, Box<dyn Error>> {
    gip.start_round(seed)?;
    if display_prompts {
        println!("--- Begin hand {:2} ---", gip.hand_num);
        println!("DeckSeed: {}", seed);
        print_player_info(gip, players, "  ");
    }
    loop {
        if matches!(gip.state, GameState::EndOfHand) {
            return Ok(false);
        }
        let p = gip.next_player().unwrap();
        let pocket = match gip.get_player_info(p).unwrap().pocket {
            None => unreachable!(),
            Some(p) => p,
        };
        let q = format!(
            "Community: {}\nPlayer {}'s action? {} {}",
            gip.table_cards
                .iter()
                .take_while(|c| c.is_some())
                .map(|c| c.unwrap().to_string())
                .collect::<Vec<_>>()
                .join(""),
            p,
            pocket[0],
            pocket[1]
        );
        match prompt(&q, display_prompts)? {
            Command::Info => {
                if display_prompts {
                    print_player_info(gip, players, "  ");
                }
            }
            Command::Quit => return Ok(true),
            Command::Help => {
                if display_prompts {
                    print_help();
                }
            }
            Command::BetAction(ba) => match gip.bet(p, ba) {
                Ok(_) => {}
                Err(e) => println!("{}", e),
            },
        }
        if display_prompts {
            println!();
        }
    }
}

fn print_test_info(gip: &GameInProgress, players: &[PlayerId]) -> Result<(), Box<dyn Error>> {
    println!("state {:?}", gip.state);
    println!("pot.total_value {}", gip.pot.total_value());
    println!("community {} {} {} {} {}",
        match gip.table_cards[0] {
            None => "None".to_string(),
            Some(c) => c.to_string(),
        },
        match gip.table_cards[1] {
            None => "None".to_string(),
            Some(c) => c.to_string(),
        },
        match gip.table_cards[2] {
            None => "None".to_string(),
            Some(c) => c.to_string(),
        },
        match gip.table_cards[3] {
            None => "None".to_string(),
            Some(c) => c.to_string(),
        },
        match gip.table_cards[4] {
            None => "None".to_string(),
            Some(c) => c.to_string(),
        },
    );
    for player in players {
        let p = gip.get_player_info(*player).expect("Must have player");
        println!("player {} bank {}", p.id, p.monies);
    }
    for player in players {
        let p = gip.get_player_info(*player).expect("Must have player");
        println!("player {} bet_status {:?}", p.id, p.bet_status);
    }
    for player in players {
        let p = gip.get_player_info(*player).expect("Must have player");
        println!("player {} pocket {}", p.id, match p.pocket {
            None => "None".to_string(),
            Some(pocket) => format!("{}{}", pocket[0], pocket[1]),
        });
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let mut gip = GameInProgress::default();
    let players: Vec<PlayerId> = (1..opt.n_players + 1).map(|i| i.into()).collect();
    for n in 1..opt.n_players + 1 {
        gip.sit_down(n.into(), opt.start_stack, n.into())?;
    }
    if !opt.no_prompts {
        println!(
            "{} players seated with {} each",
            opt.n_players, opt.start_stack
        );
    }
    loop {
        let wants_quit = single_hand(&mut gip, &players, &opt.seed, !opt.no_prompts)?;
        if wants_quit || !opt.multi_round {
            break;
        }
    }
    if !opt.no_summary {
        print_test_info(&gip, &players)?;
    }
    Ok(())
}
