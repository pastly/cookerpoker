use std::error::Error;
use std::io::{stdin, stdout, BufRead, Write};
use std::num::ParseIntError;

use poker_core::{
    deck::DeckSeed,
    game::{players::BetStatus, table::GameInProgress, BetAction, Currency},
    PlayerId,
};
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

fn prompt(q: &str) -> Result<Command, Box<dyn Error>> {
    println!("{}", q);
    let c = loop {
        print!("> ");
        stdout().flush()?;
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
            "{}{:4} Player {} [{}] {}",
            prefix,
            tokens.join("/"),
            info.id,
            info.monies,
            match info.bet_status {
                BetStatus::Folded => "Folded".to_string(),
                BetStatus::Waiting => "Waiting".to_string(),
                BetStatus::In(x) => x.to_string(),
                BetStatus::AllIn(x) => format!("{} (all in)", x),
            },
        );
    }
}

fn single_hand(
    gip: &mut GameInProgress,
    players: &[PlayerId],
    seed: &DeckSeed,
) -> Result<(), Box<dyn Error>> {
    gip.start_round(&seed)?;
    println!("--- Begin hand {:2} ---", gip.hand_num);
    println!("DeckSeed: {}", seed);
    print_player_info(gip, players, "  ");
    loop {
        match prompt("Hello")? {
            Command::Info => print_player_info(gip, players, "  "),
            Command::Quit => return Ok(()),
            Command::Help => print_help(),
            Command::BetAction(ba) => match gip.bet(1, ba) {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            },
        }
        println!();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let mut gip = GameInProgress::default();
    let players: Vec<PlayerId> = (1..opt.n_players + 1).map(|i| i.into()).collect();
    for n in 1..opt.n_players + 1 {
        gip.sit_down(n.into(), opt.start_stack, n.into())?;
    }
    println!(
        "{} players seated with {} each",
        opt.n_players, opt.start_stack
    );
    single_hand(&mut gip, &players, &opt.seed)?;
    Ok(())
}
