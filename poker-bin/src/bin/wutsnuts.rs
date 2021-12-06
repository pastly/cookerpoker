use itertools::Itertools;
use poker_core::deck::{Card, Deck, ALL_RANKS, ALL_SUITS};
use poker_core::hand::{best_of_cards, Hand};
use std::cmp::Ordering;
use std::env;
use std::io;

/// Default number of community cards to generate if no argument is given or if argument is out of
/// range or fails to parse
const DEF_COMM: usize = 3;
/// Minimum legal number of community cards
const MIN_COMM: usize = 3;
/// Maximum legal number of community cards
const MAX_COMM: usize = 5;

/// Given 3+ community cards, calculate and return the best 5-card hands and the 2-card pockets
/// that create them.
///
/// Returns: Vector of Tuples, where each tuple is a pocket ([Card; 2]) and a 5-card hand (Hand).
/// All items in the vector are equally strong, and the strongest possible pockets.
///
/// If less than three community cards are provided, returns an empty vector. In all other cases, a
/// non-empty vector is returned.
///
/// Run with --release, as with 5 community cards, this function takes a noticeable amount of time
/// to run.
///
/// This function generates and checks every possible pocket (there's only 1 deck of cards, so we
/// don't consider pockets that contain card(s) already in the community cards). For each pocket,
/// it determines the best possible 5-card combination(s) from those two cards and the community
/// cards. If they beat the current nut hands, they become the new nuts. If they tie, the nut hands
/// are extended to include them.
fn find_nuts(community: &[Card]) -> Vec<([Card; 2], Hand)> {
    let mut nuts: Vec<([Card; 2], Hand)> = vec![];
    if community.len() < 3 {
        return nuts;
    }
    // Generate a sorted deck (AAAAKKKKQQQQ...2222)
    let deck: Vec<Card> = ALL_RANKS
        .iter()
        .rev() // instead of 2->A, do A->2 so that pockets always start with higher card
        .cartesian_product(ALL_SUITS.iter().rev()) // rev again just because I like SHDC better than the reverse
        .map(|x| Card::new(*x.0, *x.1))
        .collect();
    // for every possible pocket that doesn't contain a community card ...
    for idx1 in (0..deck.len() - 1).filter(|i| !community.contains(&deck[*i])) {
        for idx2 in (idx1 + 1..deck.len()).filter(|i| !community.contains(&deck[*i])) {
            let pocket = [deck[idx1], deck[idx2]];
            // find the best 5-card hands given the 3+ community cards and the 2 pocket cards.
            // There may be more than 1 best 5-card hand.
            let mut cards = vec![deck[idx1], deck[idx2]];
            cards.extend(community);
            let best_for_pocket = best_of_cards(&cards);
            assert!(!best_for_pocket.is_empty());
            if nuts.is_empty() {
                nuts.clear();
                for h in best_for_pocket {
                    nuts.push((pocket, h));
                }
            } else {
                // hands in best_for_pocket are all equal, and all saved nuts are equal, so only
                // need to compare the first best for this pocket to the first best from saved nuts
                let best_hand = best_for_pocket[0];
                match best_hand.cmp(&nuts[0].1) {
                    Ordering::Less => {}
                    Ordering::Equal => {
                        // Equal the nut(s), so extend current nuts
                        for h in best_for_pocket {
                            nuts.push((pocket, h));
                        }
                    }
                    Ordering::Greater => {
                        // Beat the current nut(s), so replace
                        nuts.clear();
                        for h in best_for_pocket {
                            nuts.push((pocket, h));
                        }
                    }
                }
            }
        }
    }
    nuts.shrink_to_fit();
    nuts
}

fn read_and_ignore_input() {
    let stdin = io::stdin();
    let mut b = String::new();
    stdin.read_line(&mut b).unwrap();
}

fn determine_n() -> usize {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        DEF_COMM
    } else {
        let n = args[1].parse().unwrap_or(DEF_COMM);
        if (MIN_COMM..=MAX_COMM).contains(&n) {
            n
        } else {
            DEF_COMM
        }
    }
}

fn main() {
    let mut first_time_hint = " (press enter)";
    let n = determine_n();
    loop {
        let mut d = Deck::default();
        let community: Vec<Card> = (0..n).map(|_| d.draw().unwrap()).collect();
        print!("Given community cards: ");
        for c in &community {
            print!("{} ", c);
        }
        println!("the best possible hands are ...{}", first_time_hint);
        read_and_ignore_input();
        for (pocket, hand) in find_nuts(&community) {
            println!("  {}{}: {}", pocket[0], pocket[1], hand);
        }
        println!();
        if !first_time_hint.is_empty() {
            first_time_hint = "";
        }
    }
}
