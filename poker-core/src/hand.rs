use crate::deck::{Card, Rank};
use itertools::Itertools;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct Hand {
    cards: [Card; 5],
}

#[derive(Copy, Clone)]
pub enum HandClass {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

impl HandClass {
    pub fn which(hand: &Hand) -> HandClass {
        // sort a copy, in case the order of the main copy of cards is important (and also because
        // we aren't mutably borrowing the hand)
        let mut cards = hand.cards.clone();
        cards.sort_unstable();
        cards.reverse();
        if Self::is_straight_flush(&cards) {
            Self::StraightFlush
        } else if Self::is_quads(&cards) {
            Self::FourOfAKind
        } else if Self::is_full_house(&cards) {
            Self::FullHouse
        } else if Self::is_flush(&cards) {
            Self::Flush
        } else if Self::is_straight(&cards) {
            Self::Straight
        } else if Self::is_set(&cards) {
            Self::ThreeOfAKind
        } else if Self::is_two_pair(&cards) {
            Self::TwoPair
        } else if Self::is_pair(&cards) {
            Self::Pair
        } else {
            Self::HighCard
        }
    }

    fn is_straight_flush(cards: &[Card; 5]) -> bool {
        // This function requires the given cards are sorted
        Self::is_straight(cards) && Self::is_flush(cards)
    }

    fn is_quads(cards: &[Card; 5]) -> bool {
        // This function requires the given cards are sorted
        //
        // Either the first 4 cards must be the same rank, or the last 4. There's just one odd
        // card, and it must either be first or last in a sorted array of cards.
        cards[0].rank() == cards[3].rank() || cards[1].rank() == cards[4].rank()
    }

    fn is_full_house(cards: &[Card; 5]) -> bool {
        // There must only be 2 unique ranks. But that's not the only requirement: AAAA2 has two
        // ranks but isn't a full house.
        if cards.iter().map(|c| c.rank()).unique().count() != 2 {
            return false;
        }
        // There's definitely only two ranks, so it's either quads or a full house. It can't be
        // both, so just return the inverse of whether or not it's quads :)
        !Self::is_quads(cards)
    }

    fn is_straight(cards: &[Card; 5]) -> bool {
        // This function requires the given cards are sorted
        //
        // Convert ranks to ints that we can do basic math on. Rank 2 -> 0, Rank 3 -> 1, etc.
        let ints: Vec<u8> = cards
            .iter()
            .map(|c| match c.rank() {
                Rank::R2 => 0,
                Rank::R3 => 1,
                Rank::R4 => 2,
                Rank::R5 => 3,
                Rank::R6 => 4,
                Rank::R7 => 5,
                Rank::R8 => 6,
                Rank::R9 => 7,
                Rank::RT => 8,
                Rank::RJ => 9,
                Rank::RQ => 10,
                Rank::RK => 11,
                Rank::RA => 12,
            })
            .collect();
        assert_eq!(ints.len(), 5);
        // Now make sure each successive int is one less than the previous one. This is why we
        // needed the cards sorted.
        for n in 0..4 {
            if ints[n] - 1 != ints[n + 1] {
                return false;
            }
        }
        true
    }

    fn is_flush(cards: &[Card; 5]) -> bool {
        cards.iter().map(|c| c.suit()).unique().count() == 1
    }

    fn is_set(cards: &[Card; 5]) -> bool {
        // This function requires the given cards are sorted
        //
        // There must be three unique ranks, but that can either be two pair or a set and two
        // different kickers. Bail early if not 3.
        if cards.iter().map(|c| c.rank()).unique().count() != 3 {
            return false;
        }
        // Now we just need to confirm that one of the sets of 3 cards in the hand is all the same.
        // Either 1st/3rd (AAA23), 2nd/4th(ATTT2), or 3rd/5th (AK222) must be identical, because
        // the hand is sorted.
        cards[0].rank() == cards[2].rank()
            || cards[1].rank() == cards[3].rank()
            || cards[2].rank() == cards[4].rank()
    }

    fn is_two_pair(cards: &[Card; 5]) -> bool {
        // This function requires the given cards are sorted
        //
        // There must be three unique ranks, but that can either be two pair or a set and two
        // different kickers. Bail early if not 3.
        if cards.iter().map(|c| c.rank()).unique().count() != 3 {
            return false;
        }
        // There's definitely exactly 3 ranks, and it can't be both two pair and a set, so return
        // the inverse of whether it's a set :)
        !Self::is_set(cards)
    }

    fn is_pair(cards: &[Card; 5]) -> bool {
        // There's 4 ranks. 5 would be just High Card, and less than 4 would mean the hand is
        // something better.
        cards.iter().map(|c| c.rank()).unique().count() == 4
    }
}

#[derive(PartialEq, Debug)]
pub enum HandError {
    NotFiveCards(usize),
}

impl Error for HandError {}

impl fmt::Display for HandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFiveCards(n) => write!(f, "Five cards are requied, but {} were given", n),
        }
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            self.cards[0], self.cards[1], self.cards[2], self.cards[3], self.cards[4],
        )
    }
}

impl Hand {
    pub fn new(cards: &[Card]) -> Result<Self, HandError> {
        match cards.len() {
            5 => Ok(Self::new_unchecked(cards)),
            _ => Err(HandError::NotFiveCards(cards.len())),
        }
    }

    pub fn new_unchecked(c: &[Card]) -> Self {
        Self {
            cards: [c[0], c[1], c[2], c[3], c[4]],
        }
    }
}

#[cfg(test)]
mod test_hand {
    use super::*;
    use crate::deck::Deck;
    use std::iter;

    #[test]
    fn wrong_sizes() {
        let mut deck = Deck::new();
        for n in [0, 1, 2, 3, 4, 6, 7] {
            let cards: Vec<Card> = iter::repeat_with(|| deck.draw().unwrap()).take(n).collect();
            let hand = Hand::new(&cards);
            assert!(hand.is_err());
        }
    }

    #[test]
    fn correct_size() {
        let mut deck = Deck::new();
        let cards: Vec<Card> = iter::repeat_with(|| deck.draw().unwrap()).take(5).collect();
        let hand = Hand::new(&cards);
        assert!(hand.is_ok());
    }
}