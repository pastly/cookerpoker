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
const SPADE: char = 's';
const HEART: char = 'h';
const DIAMOND: char = 'd';
const CLUB: char = 'c';
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
}

impl Error for DeckError {}

impl fmt::Display for DeckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeckError::OutOfCards => write!(f, "No more cards in deck"),
        }
    }
}

#[derive(Default)]
pub struct Deck {
    cards: Vec<Card>,
    next: usize,
    infinite: bool,
}

impl Deck {
    /// Generate a new single deck of cards, shuffled
    pub fn new() -> Self {
        let mut d = Self::with_length(1);
        d.shuffle();
        d
    }

    /// Generate a deck of cards of infinite size, shuffled, and implemented as always shuffling
    /// the deck after returning a copy of the topmost card
    pub fn new_infinite() -> Self {
        // First get a non-infinite single deck
        let mut d = Self::with_length(1);
        // Set the inifinite flag
        assert!(!d.infinite);
        d.infinite = true;
        // Shuffle and return
        d.shuffle();
        d
    }

    /// Shuffle the deck of cards in-place, and reset its `next` index to 0
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
        self.next = 0;
    }

    /// Generate a new shuffled multi-deck with `l * DECK_LEN` cards
    pub fn with_length(l: usize) -> Self {
        assert!(l >= 1);
        // generate 1 Vec<Card>
        let single = {
            let mut v = Vec::with_capacity(DECK_LEN);
            for suit in ALL_SUITS.iter() {
                for rank in ALL_RANKS.iter() {
                    v.push(Card::new(*rank, *suit));
                }
            }
            v
        };
        // append copies of the single deck to the output multi-deck
        let mut multi = Vec::with_capacity(l * DECK_LEN);
        for _ in 0..l {
            multi.append(&mut single.clone());
        }
        assert_eq!(multi.len(), multi.capacity());
        let mut d = Self {
            cards: multi,
            next: 0,
            infinite: false,
        };
        // shuffle it
        d.shuffle();
        d
    }

    /// Draw the topmost card and return it, or return and error if, e.g., there are no more cards.
    pub fn draw(&mut self) -> Result<Card, DeckError> {
        if self.infinite {
            assert_eq!(self.next, 0);
            let c = self.cards[self.next];
            self.shuffle();
            Ok(c)
        } else if self.next == self.cards.len() {
            Err(DeckError::OutOfCards)
        } else {
            let c = self.cards[self.next];
            self.next += 1;
            Ok(c)
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
    fn right_len_2() {
        let d = Deck::with_length(8);
        assert_eq!(d.cards.len(), d.cards.capacity());
        assert_eq!(d.cards.len(), 8 * DECK_LEN);
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
    fn right_count_2() {
        let d = Deck::with_length(8);
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
            assert_eq!(*count, 8);
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
    fn draw_2() {
        let mut d = Deck::with_length(8);
        for _ in 0..8 * DECK_LEN {
            assert!(d.draw().is_ok());
        }
        assert_eq!(d.draw().unwrap_err(), DeckError::OutOfCards);
    }

    #[test]
    fn draw_infinite() {
        // can draw from an infinite deck many more times than its internal length. currently the
        // length is DECK_LEN, but let's further assume it's 8 * DECK_LEN and draw past that
        let mut d = Deck::new_infinite();
        for _ in 0..8 * DECK_LEN + 10 {
            assert!(d.draw().is_ok());
        }
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
}
