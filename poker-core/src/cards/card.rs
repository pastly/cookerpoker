use enum_map::Enum;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
pub const SPADE: char = 's';
pub const HEART: char = 'h';
pub const DIAMOND: char = 'd';
pub const CLUB: char = 'c';
pub const ALL_SUITS: [Suit; 4] = [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade];
pub const ALL_RANKS: [Rank; 13] = [
    Rank::Two,
    Rank::Three,
    Rank::Four,
    Rank::Five,
    Rank::Six,
    Rank::Seven,
    Rank::Eight,
    Rank::Nine,
    Rank::Ten,
    Rank::Jack,
    Rank::Queen,
    Rank::King,
    Rank::Ace,
];

#[derive(
    Hash, Enum, Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize,
)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub fn value(&self) -> u8 {
        use Rank::*;
        match *self {
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8,
            Nine => 9,
            Ten => 10,
            Jack => 11,
            Queen => 12,
            King => 13,
            Ace => 14,
        }
    }
}

impl std::fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Two => write!(f, "2"),
            Self::Three => write!(f, "3"),
            Self::Four => write!(f, "4"),
            Self::Five => write!(f, "5"),
            Self::Six => write!(f, "6"),
            Self::Seven => write!(f, "7"),
            Self::Eight => write!(f, "8"),
            Self::Nine => write!(f, "9"),
            Self::Ten => write!(f, "T"),
            Self::Jack => write!(f, "J"),
            Self::Queen => write!(f, "Q"),
            Self::King => write!(f, "K"),
            Self::Ace => write!(f, "A"),
        }
    }
}

impl From<char> for Rank {
    fn from(c: char) -> Self {
        match c {
            '2' => Rank::Two,
            '3' => Rank::Three,
            '4' => Rank::Four,
            '5' => Rank::Five,
            '6' => Rank::Six,
            '7' => Rank::Seven,
            '8' => Rank::Eight,
            '9' => Rank::Nine,
            'T' => Rank::Ten,
            'J' => Rank::Jack,
            'Q' => Rank::Queen,
            'K' => Rank::King,
            'A' => Rank::Ace,
            _ => unreachable!("Bad Rank -> Card Parse"),
        }
    }
}

// Not intended to be pub
impl From<Rank> for i8 {
    fn from(r: Rank) -> Self {
        match r {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 12,
            Rank::King => 13,
            Rank::Ace => 14,
        }
    }
}

#[derive(Hash, Enum, Clone, Copy, Debug, PartialEq, Eq, Ord, Serialize, Deserialize)]
pub enum Suit {
    Club,
    Diamond,
    Heart,
    Spade,
}

impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Club => write!(f, "{}", CLUB),
            Self::Diamond => write!(f, "{}", DIAMOND),
            Self::Heart => write!(f, "{}", HEART),
            Self::Spade => write!(f, "{}", SPADE),
        }
    }
}

impl From<char> for Suit {
    fn from(c: char) -> Self {
        match c {
            CLUB => Self::Club,
            DIAMOND => Self::Diamond,
            HEART => Self::Heart,
            SPADE => Self::Spade,
            _ => unreachable!("Bad Suit -> Card parse"),
        }
    }
}
/// All suits are equal
impl PartialOrd for Suit {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

impl FromStr for Card {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        assert_eq!(s.len(), 2);
        let mut i = s.chars();
        Ok(Card::from([
            i.next().ok_or(String::from("Failed to parse card"))?,
            i.next().ok_or(String::from("Failed to parse card"))?,
        ]))
    }
}

impl From<[char; 2]> for Card {
    fn from(cs: [char; 2]) -> Self {
        Self {
            rank: cs[0].into(),
            suit: cs[1].into(),
        }
    }
}
/// We only consider Card Rank when determining order
impl std::cmp::PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.rank.partial_cmp(&other.rank)
    }
}

/// We only consider Card Rank when determining order
impl std::cmp::Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank.cmp(&other.rank)
    }
}

impl Card {
    // Manual eq function that ignores suit
    pub fn eq(&self, other: &Card) -> bool {
        self.rank == other.rank
    }

    pub const fn new(suit: Suit, rank: Rank) -> Self {
        Card { rank, suit }
    }
}

/// Returns an UNSHUFFLED array of cards
pub fn all_cards() -> [Card; 52] {
    use itertools::Itertools;
    // Default value, can probably unsafe this if it isn't optimized well
    let mut cards: [Card; 52] = [Card::new(Suit::Club, Rank::Ace); 52];
    let c_iter = ALL_SUITS
        .iter()
        .cartesian_product(ALL_RANKS.iter())
        .map(|x| Card::new(*x.0, *x.1));
    for (i, c) in c_iter.enumerate() {
        cards[i] = c;
    }
    cards
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    /// Becuase the sort order of cards is used as logic, this test simply
    /// exists to highlight when that fails
    fn sort_order() {
        for (i, r) in ALL_RANKS.into_iter().sorted_unstable().rev().enumerate() {
            assert_eq!(r.value(), 14u8 - (i as u8));
        }
    }

    #[test]
    fn string_single() {
        let mut s = "Ah".chars().into_iter();
        let ch = [s.next().unwrap(), s.next().unwrap()];
        let c = Card::from(ch);
        assert_eq!(c.rank, Rank::Ace);
        assert_eq!(c.suit, Suit::Heart);
    }

    #[test]
    fn test_card_rank() {
        let c1 = Card::new(Suit::Club, Rank::Jack);
        let c2 = Card::new(Suit::Diamond, Rank::Queen);
        let c3 = Card::new(Suit::Heart, Rank::Jack);
        assert!(c1 < c2);
        assert!(c1.eq(&c3));
    }
}
