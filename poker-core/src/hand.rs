use crate::deck::{Card, Rank};
use itertools::Itertools;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct Hand {
    cards: [Card; 5],
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
        //
        // It's important that the order of these checks is maintained from best-hand to
        // worst-hand. The check for hand type $foo only verifies the hand can be considered $foo,
        // not that $foo is the best thing it can be considered. I can only think of one example,
        // unfortunately. It is: is_straight() doesn't check if the hand is also a flush, thus
        // is_straight_flush() must be called first.
        let mut cards = hand.cards;
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
        let ints: Vec<i8> = cards
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
        // Check specifically for A2345 straight, as it will appear as A5432 (aka 12, 3, 2, 1, 0)
        // and not look like a straight.
        if ints == [12, 3, 2, 1, 0] {
            return true;
        }
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

#[cfg(test)]
mod test_hand_class {
    use super::*;
    use crate::deck::{Rank, Suit};

    const ALL_RANKS: [Rank; 13] = [
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
    const ALL_SUITS: [Suit; 4] = [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade];

    // All the straight flushes are correctly identified as such.
    #[test]
    fn straight_flushes() {
        for ranks in [
            [Rank::RA, Rank::RK, Rank::RQ, Rank::RJ, Rank::RT],
            [Rank::RK, Rank::RQ, Rank::RJ, Rank::RT, Rank::R9],
            [Rank::RQ, Rank::RJ, Rank::RT, Rank::R9, Rank::R8],
            [Rank::RJ, Rank::RT, Rank::R9, Rank::R8, Rank::R7],
            [Rank::RT, Rank::R9, Rank::R8, Rank::R7, Rank::R6],
            [Rank::R9, Rank::R8, Rank::R7, Rank::R6, Rank::R5],
            [Rank::R8, Rank::R7, Rank::R6, Rank::R5, Rank::R4],
            [Rank::R7, Rank::R6, Rank::R5, Rank::R4, Rank::R3],
            [Rank::R6, Rank::R5, Rank::R4, Rank::R3, Rank::R2],
            [Rank::R5, Rank::R4, Rank::R3, Rank::R2, Rank::RA],
        ] {
            for suit in ALL_SUITS {
                let hand = Hand::new_unchecked(&[
                    Card::new(ranks[0], suit),
                    Card::new(ranks[1], suit),
                    Card::new(ranks[2], suit),
                    Card::new(ranks[3], suit),
                    Card::new(ranks[4], suit),
                ]);
                assert_eq!(HandClass::which(&hand), HandClass::StraightFlush);
            }
        }
    }

    // Test all quads (but not with all kickers)
    #[test]
    fn quads() {
        for rank in ALL_RANKS {
            let extra = Card::new(
                match rank {
                    Rank::R2 => Rank::R3,
                    _ => Rank::R2,
                },
                Suit::Club,
            );
            let hand = Hand::new_unchecked(&[
                Card::new(rank, Suit::Club),
                Card::new(rank, Suit::Diamond),
                Card::new(rank, Suit::Heart),
                Card::new(rank, Suit::Spade),
                extra,
            ]);
            assert_eq!(HandClass::which(&hand), HandClass::FourOfAKind);
        }
    }

    // All combinations of 2 ranks in a full house, but not with all combos of suit too
    #[test]
    fn boat() {
        for rank3 in ALL_RANKS {
            for rank2 in ALL_RANKS {
                if rank2 == rank3 {
                    continue;
                }
                let hand = Hand::new_unchecked(&[
                    Card::new(rank3, Suit::Club),
                    Card::new(rank3, Suit::Diamond),
                    Card::new(rank3, Suit::Heart),
                    Card::new(rank2, Suit::Club),
                    Card::new(rank2, Suit::Diamond),
                ]);
                assert_eq!(HandClass::which(&hand), HandClass::FullHouse);
            }
        }
    }

    // A couple arbitrarily chosen 5 card hands, but all suits
    #[test]
    fn flush() {
        for ranks in [
            [Rank::RA, Rank::RK, Rank::RQ, Rank::RJ, Rank::R2],
            [Rank::RT, Rank::R8, Rank::R6, Rank::R4, Rank::R2],
            [Rank::R2, Rank::R4, Rank::R5, Rank::R6, Rank::R7],
        ] {
            for suit in ALL_SUITS {
                let hand = Hand::new_unchecked(&[
                    Card::new(ranks[0], suit),
                    Card::new(ranks[1], suit),
                    Card::new(ranks[2], suit),
                    Card::new(ranks[3], suit),
                    Card::new(ranks[4], suit),
                ]);
                assert_eq!(HandClass::which(&hand), HandClass::Flush);
            }
        }
    }

    #[test]
    fn straight() {
        for ranks in [
            [Rank::RA, Rank::RK, Rank::RQ, Rank::RJ, Rank::RT],
            [Rank::RK, Rank::RQ, Rank::RJ, Rank::RT, Rank::R9],
            [Rank::RQ, Rank::RJ, Rank::RT, Rank::R9, Rank::R8],
            [Rank::RJ, Rank::RT, Rank::R9, Rank::R8, Rank::R7],
            [Rank::RT, Rank::R9, Rank::R8, Rank::R7, Rank::R6],
            [Rank::R9, Rank::R8, Rank::R7, Rank::R6, Rank::R5],
            [Rank::R8, Rank::R7, Rank::R6, Rank::R5, Rank::R4],
            [Rank::R7, Rank::R6, Rank::R5, Rank::R4, Rank::R3],
            [Rank::R6, Rank::R5, Rank::R4, Rank::R3, Rank::R2],
            [Rank::R5, Rank::R4, Rank::R3, Rank::R2, Rank::RA],
        ] {
            let hand = Hand::new_unchecked(&[
                Card::new(ranks[0], Suit::Club),
                Card::new(ranks[1], Suit::Club),
                Card::new(ranks[2], Suit::Club),
                Card::new(ranks[3], Suit::Club),
                Card::new(ranks[4], Suit::Spade),
            ]);
            assert_eq!(HandClass::which(&hand), HandClass::Straight);
        }
    }

    #[test]
    fn set() {
        for rank in ALL_RANKS {
            let r2 = match rank {
                Rank::R2 => Rank::R3,
                _ => Rank::R2,
            };
            let r3 = match rank {
                Rank::RA => Rank::RK,
                _ => Rank::RA,
            };
            let hand = Hand::new_unchecked(&[
                Card::new(rank, Suit::Club),
                Card::new(rank, Suit::Diamond),
                Card::new(rank, Suit::Heart),
                Card::new(r2, Suit::Club),
                Card::new(r3, Suit::Club),
            ]);
            assert_eq!(HandClass::which(&hand), HandClass::ThreeOfAKind);
        }
    }

    #[test]
    fn two_pair() {
        for r1 in ALL_RANKS {
            for r2 in ALL_RANKS {
                if r1 == r2 {
                    continue;
                }
                let r3 = if r1 != Rank::RA && r2 != Rank::RA {
                    Rank::RA
                } else if r1 != Rank::RK && r2 != Rank::RK {
                    Rank::RK
                } else {
                    Rank::RQ
                };
                let hand = Hand::new_unchecked(&[
                    Card::new(r1, Suit::Club),
                    Card::new(r1, Suit::Diamond),
                    Card::new(r2, Suit::Club),
                    Card::new(r2, Suit::Diamond),
                    Card::new(r3, Suit::Spade),
                ]);
                assert_eq!(HandClass::which(&hand), HandClass::TwoPair);
            }
        }
    }

    #[test]
    fn pair() {
        for rank in ALL_RANKS {
            let r1 = match rank {
                Rank::R2 => Rank::R3,
                _ => Rank::R2,
            };
            let r2 = match rank {
                Rank::R4 => Rank::R5,
                _ => Rank::R4,
            };
            let r3 = match rank {
                Rank::R6 => Rank::R7,
                _ => Rank::R6,
            };
            let hand = Hand::new_unchecked(&[
                Card::new(r1, Suit::Club),
                Card::new(r2, Suit::Club),
                Card::new(r3, Suit::Club),
                Card::new(rank, Suit::Club),
                Card::new(rank, Suit::Diamond),
            ]);
            assert_eq!(HandClass::which(&hand), HandClass::Pair);
        }
    }

    #[test]
    fn high_card() {
        for ranks in [
            [Rank::RA, Rank::RK, Rank::RQ, Rank::RJ, Rank::R2],
            [Rank::RT, Rank::R8, Rank::R6, Rank::R4, Rank::R2],
            [Rank::R2, Rank::R4, Rank::R5, Rank::R6, Rank::R7],
        ] {
            let hand = Hand::new_unchecked(&[
                Card::new(ranks[0], Suit::Club),
                Card::new(ranks[1], Suit::Club),
                Card::new(ranks[2], Suit::Club),
                Card::new(ranks[3], Suit::Club),
                Card::new(ranks[4], Suit::Diamond),
            ]);
            assert_eq!(HandClass::which(&hand), HandClass::HighCard);
        }
    }
}
