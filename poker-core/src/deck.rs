use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub const ALL_RANKS: [Rank; 13] = [
    Rank::R2,
    Rank::R3,
    Rank::R4,
    Rank::R5,
    Rank::R6,
    Rank::R7,
    Rank::R8,
    Rank::R9,
    Rank::RT,
    Rank::RJ,
    Rank::RQ,
    Rank::RK,
    Rank::RA,
];
pub const ALL_SUITS: [Suit; 4] = [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade];
const DECK_LEN: usize = ALL_RANKS.len() * ALL_SUITS.len();
pub const SPADE: char = 's';
pub const HEART: char = 'h';
pub const DIAMOND: char = 'd';
pub const CLUB: char = 'c';
/// TECHNICALLY this could be 22.
/// 22x2(pockets)+3(burn)+5(table) = DECK_LEN
pub const MAX_PLAYERS: u8 = 21;
//const SPADE: &str = "♤";
//const HEART: &str = "♡";
//const DIAMOND: &str = "♢";
//const CLUB: &str = "♧";
//const SPADE: &str = "♠";
//const HEART: &str = "♥";
//const DIAMOND: &str = "♦";
//const CLUB: &str = "♣";

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Suit {
    Club,
    Diamond,
    Heart,
    Spade,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Club => write!(f, "{}", CLUB),
            Self::Diamond => write!(f, "{}", DIAMOND),
            Self::Heart => write!(f, "{}", HEART),
            Self::Spade => write!(f, "{}", SPADE),
        }
    }
}

#[cfg(test)]
impl From<char> for Suit {
    fn from(c: char) -> Self {
        match c {
            CLUB => Self::Club,
            DIAMOND => Self::Diamond,
            HEART => Self::Heart,
            SPADE => Self::Spade,
            _ => unreachable!(),
        }
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Rank {
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    RT,
    RJ,
    RQ,
    RK,
    RA,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::R2 => write!(f, "2"),
            Self::R3 => write!(f, "3"),
            Self::R4 => write!(f, "4"),
            Self::R5 => write!(f, "5"),
            Self::R6 => write!(f, "6"),
            Self::R7 => write!(f, "7"),
            Self::R8 => write!(f, "8"),
            Self::R9 => write!(f, "9"),
            Self::RT => write!(f, "T"),
            Self::RJ => write!(f, "J"),
            Self::RQ => write!(f, "Q"),
            Self::RK => write!(f, "K"),
            Self::RA => write!(f, "A"),
        }
    }
}

#[cfg(test)]
impl From<char> for Rank {
    fn from(c: char) -> Self {
        match c {
            '2' => Rank::R2,
            '3' => Rank::R3,
            '4' => Rank::R4,
            '5' => Rank::R5,
            '6' => Rank::R6,
            '7' => Rank::R7,
            '8' => Rank::R8,
            '9' => Rank::R9,
            'T' => Rank::RT,
            'J' => Rank::RJ,
            'Q' => Rank::RQ,
            'K' => Rank::RK,
            'A' => Rank::RA,
            _ => unreachable!(),
        }
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Card {
    rank: Rank,
    suit: Suit,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

#[cfg(test)]
impl From<[char; 2]> for Card {
    fn from(cs: [char; 2]) -> Self {
        Self {
            rank: cs[0].into(),
            suit: cs[1].into(),
        }
    }
}

#[cfg(test)]
pub fn cards_from_str(s: &'static str) -> Vec<Card> {
    let mut v = vec![];
    let mut s_chars = s.chars();
    while let Some(r) = s_chars.next() {
        let s = s_chars.next().expect("Need even number of chars");
        v.push([r, s].into())
    }
    v
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }

    pub fn suit(self) -> Suit {
        self.suit
    }

    pub fn rank(self) -> Rank {
        self.rank
    }
}

#[derive(PartialEq, Debug)]
pub enum DeckError {
    OutOfCards,
    TooManyPlayers,
}

impl Error for DeckError {}

impl fmt::Display for DeckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeckError::OutOfCards => write!(f, "No more cards in deck"),
            &DeckError::TooManyPlayers => write!(f, "Too many players to deal"),
        }
    }
}

#[derive(Default)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    /// Generate a new single deck of cards, shuffled
    pub fn new() -> Self {
        use itertools::Itertools;
        let c: Vec<Card> = ALL_RANKS
            .iter()
            .cartesian_product(ALL_SUITS.iter())
            .map(|x| Card::new(*x.0, *x.1))
            .collect();
        assert_eq!(c.len(), DECK_LEN);
        let mut d = Deck { cards: c };
        d.shuffle();
        d
    }

    /// Shuffle the deck of cards in-place, and reset its `next` index to 0
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }

    /// Draw the topmost card and return it, or return and error if, e.g., there are no more cards.
    pub fn draw(&mut self) -> Result<Card, DeckError> {
        self.cards.pop().ok_or(DeckError::OutOfCards)
    }

    pub fn burn(&mut self) {
        self.cards.pop();
    }

    pub fn deal_pockets(&mut self, num_players: u8) -> Result<Vec<[Card; 2]>, DeckError> {
        if num_players > MAX_PLAYERS {
            return Err(DeckError::TooManyPlayers);
        } else {
            let mut v = Vec::new();
            // Range only works in positive direction
            for i in (1..=num_players).rev() {
                let c1 = self.draw()?;
                let c2 = self.cards.remove(self.cards.len() - i as usize);
                v.push([c1, c2])
            }
            Ok(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn right_len_1() {
        let d = Deck::new();
        assert_eq!(d.cards.len(), d.cards.capacity());
        assert_eq!(d.cards.len(), DECK_LEN);
    }

    #[test]
    fn right_count_1() {
        let d = Deck::new();
        let mut counts: HashMap<Card, u16> = HashMap::new();
        for card in d.cards.iter() {
            if let Some(count) = counts.get_mut(card) {
                *count += 1;
            } else {
                counts.insert(*card, 1);
            }
        }
        assert_eq!(counts.len(), DECK_LEN);
        for count in counts.values() {
            assert_eq!(*count, 1);
        }
    }

    #[test]
    fn draw_1() {
        let mut d = Deck::new();
        for _ in 0..DECK_LEN {
            assert!(d.draw().is_ok());
        }
        assert_eq!(d.draw().unwrap_err(), DeckError::OutOfCards);
    }

    #[test]
    fn string_empty() {
        let s = "";
        let res = cards_from_str(&s);
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn string_single() {
        let s = "Ah";
        let res = cards_from_str(&s);
        assert_eq!(res.len(), 1);
        let c = res[0];
        assert_eq!(c.rank(), Rank::RA);
        assert_eq!(c.suit(), Suit::Heart);
    }

    #[test]
    fn string_multi() {
        let s = "Ah2c6h";
        let res = cards_from_str(&s);
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn is_shuffled() {
        let mut d = Deck::new();
        let top = d.draw().unwrap();
        let next = d.draw().unwrap();
        let third = d.draw().unwrap();
        let fourth = d.draw().unwrap();
        if top.rank() == Rank::RA
            && next.rank() == Rank::RA
            && third.rank() == Rank::RA
            && fourth.rank() == Rank::RA
        {
            panic!("Top four cards were all aces! This indicates the deck was not shuffled. There is a *very* small chance this is a false positive.")
        }
    }

    #[test]
    fn deal_pockets_1() {
        let mut d = Deck::new();
        let expect = [d.cards[51], d.cards[50]];
        let actual = d.deal_pockets(1).unwrap();
        assert_eq!(actual[0], expect);
    }

    #[test]
    fn deal_pockets_2() {
        let mut d = Deck::new();
        println!("46 {}", d.cards[46]);
        println!("47 {}", d.cards[47]);
        println!("48 {}", d.cards[48]);
        println!("49 {}", d.cards[49]);
        println!("50 {}", d.cards[50]);
        println!("51 {}", d.cards[51]);
        let expect1 = [d.cards[51], d.cards[49]];
        let expect2 = [d.cards[50], d.cards[48]];
        let actual = d.deal_pockets(2).unwrap();
        assert_eq!(actual[0], expect1);
        assert_eq!(actual[1], expect2);
    }

    #[test]
    fn deal_pockets() {
        let mut d = Deck::new();
        let v = d.deal_pockets(10).expect("Can't deal pockets?");
        assert_eq!(d.cards.len(), DECK_LEN - 20);
        assert_eq!(v.len(), 10);
    }
}
