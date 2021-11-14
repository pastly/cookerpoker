use crate::deck::{Card, Rank};
use itertools::{zip, Itertools};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum WinState {
    Win,
    Tie,
    Lose,
}

impl From<Ordering> for WinState {
    fn from(o: Ordering) -> Self {
        match o {
            Ordering::Less => WinState::Lose,
            Ordering::Greater => WinState::Win,
            Ordering::Equal => WinState::Tie,
        }
    }
}

#[derive(Debug)]
pub struct Hand {
    cards: [Card; 5],
    class: HandClass,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    fn beats(c1: &[Card], c2: &[Card]) -> WinState {
        let hc1 = HandClass::which(c1);
        let hc2 = HandClass::which(c2);
        match hc1.cmp(&hc2) {
            Ordering::Equal => {}
            o => return o.into(),
        };
        assert_eq!(hc1, hc2);
        let mut left: [Rank; 5] = [
            c1[0].rank(),
            c1[1].rank(),
            c1[2].rank(),
            c1[3].rank(),
            c1[4].rank(),
        ];
        let mut right: [Rank; 5] = [
            c2[0].rank(),
            c2[1].rank(),
            c2[2].rank(),
            c2[3].rank(),
            c2[4].rank(),
        ];
        left.sort_unstable();
        left.reverse();
        right.sort_unstable();
        right.reverse();
        match hc1 {
            HandClass::StraightFlush => HandClass::beats_straight_flush(left, right),
            HandClass::FourOfAKind => HandClass::beats_quads(left, right),
            HandClass::FullHouse => HandClass::beats_full_house(left, right),
            HandClass::Flush => HandClass::beats_flush(left, right),
            HandClass::Straight => HandClass::beats_straight(left, right),
            HandClass::ThreeOfAKind => HandClass::beats_set(left, right),
            HandClass::TwoPair => HandClass::beats_two_pair(left, right),
            HandClass::Pair => HandClass::beats_pair(left, right),
            HandClass::HighCard => HandClass::beats_high_card(left, right),
        }
        .into()
    }

    fn beats_straight_flush(left: [Rank; 5], right: [Rank; 5]) -> Ordering {
        // flush part is equal; only need to compare the straight part
        Self::beats_straight(left, right)
    }

    fn beats_quads(left: [Rank; 5], right: [Rank; 5]) -> Ordering {
        // the quads will either be 0-3 or 1-4, and kicker the remainder
        let (quad1, kick1) = if left[0] == left[3] {
            (left[0], left[4])
        } else {
            (left[4], left[0])
        };
        let (quad2, kick2) = if right[0] == right[3] {
            (right[0], right[4])
        } else {
            (right[4], right[0])
        };
        match quad1.cmp(&quad2) {
            Ordering::Equal => kick1.cmp(&kick2),
            o => o,
        }
    }

    fn beats_full_house(left: [Rank; 5], right: [Rank; 5]) -> Ordering {
        // The logic is the same as for beats_set(), except both "kickers" in a hand are the same
        Self::beats_set(left, right)
    }

    fn beats_flush(left: [Rank; 5], right: [Rank; 5]) -> Ordering {
        Self::beats_high_card(left, right)
    }

    fn beats_straight(left: [Rank; 5], right: [Rank; 5]) -> Ordering {
        // have to look special at 5432A straight, as it will be A5432 since cards are sorted by
        // rank.
        let l = match (left[0], left[1]) {
            (Rank::RA, Rank::R5) => Rank::R5,
            (first, _) => first,
        };
        let r = match (right[0], right[1]) {
            (Rank::RA, Rank::R5) => Rank::R5,
            (first, _) => first,
        };
        l.cmp(&r)
    }

    fn beats_set(left: [Rank; 5], right: [Rank; 5]) -> Ordering {
        // The set is either 0-2, 1-3, or 2-4. The kickers ar ethe remainder
        let (trio1, kick1) = if left[0] == left[2] {
            (left[0], (left[3], left[4]))
        } else if left[1] == left[3] {
            (left[1], (left[0], left[4]))
        } else {
            (left[2], (left[0], left[1]))
        };
        let (trio2, kick2) = if right[0] == right[2] {
            (right[0], (right[3], right[4]))
        } else if right[1] == right[3] {
            (right[1], (right[0], right[4]))
        } else {
            (right[2], (right[0], right[1]))
        };
        // Yes, with a single deck the set should never be the same for both hands. But for "future
        // proofing", I'm goign to check anyway.
        match trio1.cmp(&trio2) {
            Ordering::Equal => match kick1.0.cmp(&kick2.0) {
                Ordering::Equal => kick1.1.cmp(&kick2.1),
                o => o,
            },
            o => o,
        }
    }

    fn beats_two_pair(left: [Rank; 5], right: [Rank; 5]) -> Ordering {
        // find the two pairs by finding the odd ball card instead.
        // If it's 0th, then 1-2 and 3-4 are the pairs.
        // if it's 4th, then 0-1 and 2-3 are the pairs.
        // If it's 2nd, then 0-1 and 3-4 are the pairs.
        let (pairs1, kick1) = if left[0] != left[1] {
            ((left[1], left[3]), left[0])
        } else if left[4] != left[3] {
            ((left[0], left[2]), left[4])
        } else {
            ((left[0], left[3]), left[2])
        };
        let (pairs2, kick2) = if right[0] != right[1] {
            ((right[1], right[3]), right[0])
        } else if right[4] != right[3] {
            ((right[0], right[2]), right[4])
        } else {
            ((right[0], right[3]), right[2])
        };
        match pairs1.0.cmp(&pairs2.0) {
            Ordering::Equal => match pairs1.1.cmp(&pairs2.1) {
                Ordering::Equal => kick1.cmp(&kick2),
                o => o,
            },
            o => o,
        }
    }

    fn beats_pair(left: [Rank; 5], right: [Rank; 5]) -> Ordering {
        let (pair1, kick1) = if left[0] == left[1] {
            (left[0], (left[2], left[3], left[4]))
        } else if left[1] == left[2] {
            (left[1], (left[0], left[3], left[4]))
        } else if left[2] == left[3] {
            (left[2], (left[0], left[1], left[4]))
        } else {
            (left[3], (left[0], left[1], left[2]))
        };
        let (pair2, kick2) = if right[0] == right[1] {
            (right[0], (right[2], right[3], right[4]))
        } else if right[1] == right[2] {
            (right[1], (right[0], right[3], right[4]))
        } else if right[2] == right[3] {
            (right[2], (right[0], right[1], right[4]))
        } else {
            (right[3], (right[0], right[1], right[2]))
        };
        match pair1.cmp(&pair2) {
            Ordering::Equal => match kick1.0.cmp(&kick2.0) {
                Ordering::Equal => match kick1.1.cmp(&kick2.1) {
                    Ordering::Equal => kick1.2.cmp(&kick2.2),
                    o => o,
                },
                o => o,
            },
            o => o,
        }
    }

    fn beats_high_card(left: [Rank; 5], right: [Rank; 5]) -> Ordering {
        for (l, r) in zip(left.iter(), right.iter()) {
            match l.cmp(r) {
                Ordering::Equal => {}
                o => return o,
            };
        }
        Ordering::Equal
    }

    fn which(c: &[Card]) -> HandClass {
        // sort a copy, in case the order of the main copy of cards is important (and also because
        // we aren't mutably borrowing the hand)
        //
        // It's important that the order of these checks is maintained from best-hand to
        // worst-hand. The check for hand type $foo only verifies the hand can be considered $foo,
        // not that $foo is the best thing it can be considered. I can only think of one example,
        // unfortunately. It is: is_straight() doesn't check if the hand is also a flush, thus
        // is_straight_flush() must be called first.
        assert_eq!(c.len(), 5);
        let mut cards: [Card; 5] = [c[0], c[1], c[2], c[3], c[4]];
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
            class: HandClass::which(c),
        }
    }

    pub fn beats(&self, other: &Self) -> WinState {
        match self.class.cmp(&other.class) {
            Ordering::Equal => HandClass::beats(&self.cards, &other.cards),
            o => o.into(),
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
    use crate::deck::{cards_from_str, Rank, Suit};

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
                let cards = [
                    Card::new(ranks[0], suit),
                    Card::new(ranks[1], suit),
                    Card::new(ranks[2], suit),
                    Card::new(ranks[3], suit),
                    Card::new(ranks[4], suit),
                ];
                assert_eq!(HandClass::which(&cards), HandClass::StraightFlush);
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
            let cards = [
                Card::new(rank, Suit::Club),
                Card::new(rank, Suit::Diamond),
                Card::new(rank, Suit::Heart),
                Card::new(rank, Suit::Spade),
                extra,
            ];
            assert_eq!(HandClass::which(&cards), HandClass::FourOfAKind);
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
                let cards = [
                    Card::new(rank3, Suit::Club),
                    Card::new(rank3, Suit::Diamond),
                    Card::new(rank3, Suit::Heart),
                    Card::new(rank2, Suit::Club),
                    Card::new(rank2, Suit::Diamond),
                ];
                assert_eq!(HandClass::which(&cards), HandClass::FullHouse);
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
                let cards = [
                    Card::new(ranks[0], suit),
                    Card::new(ranks[1], suit),
                    Card::new(ranks[2], suit),
                    Card::new(ranks[3], suit),
                    Card::new(ranks[4], suit),
                ];
                assert_eq!(HandClass::which(&cards), HandClass::Flush);
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
            let cards = [
                Card::new(ranks[0], Suit::Club),
                Card::new(ranks[1], Suit::Club),
                Card::new(ranks[2], Suit::Club),
                Card::new(ranks[3], Suit::Club),
                Card::new(ranks[4], Suit::Spade),
            ];
            assert_eq!(HandClass::which(&cards), HandClass::Straight);
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
            let cards = [
                Card::new(rank, Suit::Club),
                Card::new(rank, Suit::Diamond),
                Card::new(rank, Suit::Heart),
                Card::new(r2, Suit::Club),
                Card::new(r3, Suit::Club),
            ];
            assert_eq!(HandClass::which(&cards), HandClass::ThreeOfAKind);
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
                let cards = [
                    Card::new(r1, Suit::Club),
                    Card::new(r1, Suit::Diamond),
                    Card::new(r2, Suit::Club),
                    Card::new(r2, Suit::Diamond),
                    Card::new(r3, Suit::Spade),
                ];
                assert_eq!(HandClass::which(&cards), HandClass::TwoPair);
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
            let cards = [
                Card::new(r1, Suit::Club),
                Card::new(r2, Suit::Club),
                Card::new(r3, Suit::Club),
                Card::new(rank, Suit::Club),
                Card::new(rank, Suit::Diamond),
            ];
            assert_eq!(HandClass::which(&cards), HandClass::Pair);
        }
    }

    #[test]
    fn high_card() {
        for ranks in [
            [Rank::RA, Rank::RK, Rank::RQ, Rank::RJ, Rank::R2],
            [Rank::RT, Rank::R8, Rank::R6, Rank::R4, Rank::R2],
            [Rank::R2, Rank::R4, Rank::R5, Rank::R6, Rank::R7],
        ] {
            let cards = [
                Card::new(ranks[0], Suit::Club),
                Card::new(ranks[1], Suit::Club),
                Card::new(ranks[2], Suit::Club),
                Card::new(ranks[3], Suit::Club),
                Card::new(ranks[4], Suit::Diamond),
            ];
            assert_eq!(HandClass::which(&cards), HandClass::HighCard);
        }
    }

    #[test]
    fn beats_straight_flush_tie() {
        let h1 = Hand::new_unchecked(&cards_from_str("AcKcQcJcTc"));
        let h2 = Hand::new_unchecked(&cards_from_str("AdKdQdJdTd"));
        assert_eq!(h1.beats(&h2), WinState::Tie);
        let h1 = Hand::new_unchecked(&cards_from_str("KcQcJcTc9c"));
        let h2 = Hand::new_unchecked(&cards_from_str("KdQdJdTd9d"));
        assert_eq!(h1.beats(&h2), WinState::Tie);
        let h1 = Hand::new_unchecked(&cards_from_str("5c4c3c2cAc"));
        let h2 = Hand::new_unchecked(&cards_from_str("5d4d3d2dAd"));
        assert_eq!(h1.beats(&h2), WinState::Tie);
    }

    #[test]
    fn beats_straight_flush_win() {
        let h1 = Hand::new_unchecked(&cards_from_str("AcKcQcJcTc"));
        let h2 = Hand::new_unchecked(&cards_from_str("KdQdJdTd9d"));
        println!("{} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Win);
        let h1 = Hand::new_unchecked(&cards_from_str("6c5c4c3c2c"));
        let h2 = Hand::new_unchecked(&cards_from_str("5d4d3d2dAd"));
        println!("{} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Win);
        let h1 = Hand::new_unchecked(&cards_from_str("AcKcQcJcTc"));
        let h2 = Hand::new_unchecked(&cards_from_str("5d4d3d2dAd"));
        println!("{} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Win);
    }

    #[test]
    fn beats_straight_flush_lose() {
        let h1 = Hand::new_unchecked(&cards_from_str("KdQdJdTd9d"));
        let h2 = Hand::new_unchecked(&cards_from_str("AcKcQcJcTc"));
        println!("{} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Lose);
        let h1 = Hand::new_unchecked(&cards_from_str("5d4d3d2dAd"));
        let h2 = Hand::new_unchecked(&cards_from_str("6c5c4c3c2c"));
        println!("{} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Lose);
    }

    #[test]
    fn beats_quads_tie() {
        // this should be impossible in typical single deck poker, but check for it anyway since
        // the logic doesn't care
        let h1 = Hand::new_unchecked(&cards_from_str("2c2d2h2s3c"));
        let h2 = Hand::new_unchecked(&cards_from_str("2c2d2h2s3d"));
        println!("{} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Tie);
    }

    #[test]
    fn beats_quads_win() {
        let h1 = Hand::new_unchecked(&cards_from_str("4c4d4h4s3c"));
        let h2 = Hand::new_unchecked(&cards_from_str("3c3d3h3s2d"));
        println!("{} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Win);
        let h1 = Hand::new_unchecked(&cards_from_str("4c4d4h4s5c"));
        let h2 = Hand::new_unchecked(&cards_from_str("4c4d4h4s3c"));
        println!("{} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Win);
    }

    #[test]
    fn beats_quads_lose() {
        let h1 = Hand::new_unchecked(&cards_from_str("3c3d3h3s2d"));
        let h2 = Hand::new_unchecked(&cards_from_str("4c4d4h4s3c"));
        println!("{} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Lose);
        let h1 = Hand::new_unchecked(&cards_from_str("4c4d4h4s3c"));
        let h2 = Hand::new_unchecked(&cards_from_str("4c4d4h4s5c"));
        println!("{} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Lose);
    }

    #[test]
    fn beats_full_house_tie() {
        unimplemented!()
    }

    #[test]
    fn beats_full_house_win() {
        unimplemented!()
    }

    #[test]
    fn beats_full_house_lose() {
        unimplemented!()
    }

    #[test]
    fn beats_flush_tie() {
        unimplemented!()
    }

    #[test]
    fn beats_flush_win() {
        unimplemented!()
    }

    #[test]
    fn beats_flush_lose() {
        unimplemented!()
    }

    #[test]
    fn beats_straight_tie() {
        unimplemented!()
    }

    #[test]
    fn beats_straight_win() {
        unimplemented!()
    }

    #[test]
    fn beats_straight_lose() {
        unimplemented!()
    }

    #[test]
    fn beats_set_tie() {
        unimplemented!()
    }

    #[test]
    fn beats_set_win() {
        unimplemented!()
    }

    #[test]
    fn beats_set_lose() {
        unimplemented!()
    }

    #[test]
    fn beats_two_pair_tie() {
        unimplemented!()
    }

    #[test]
    fn beats_two_pair_win() {
        unimplemented!()
    }

    #[test]
    fn beats_two_pair_lose() {
        unimplemented!()
    }

    #[test]
    fn beats_pair_tie() {
        unimplemented!()
    }

    #[test]
    fn beats_pair_win() {
        unimplemented!()
    }

    #[test]
    fn beats_pair_lose() {
        unimplemented!()
    }

    #[test]
    fn beats_high_card_tie() {
        unimplemented!()
    }

    #[test]
    fn beats_high_card_win() {
        unimplemented!()
    }

    #[test]
    fn beats_high_card_lose() {
        unimplemented!()
    }
}
