use crate::deck::{Card, Rank};
use itertools::{zip, Itertools};
use std::cmp::Ordering;
use std::collections::HashMap;
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

impl From<WinState> for Ordering {
    fn from(ws: WinState) -> Self {
        match ws {
            WinState::Lose => Ordering::Less,
            WinState::Win => Ordering::Greater,
            WinState::Tie => Ordering::Equal,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    NotTwoCards(usize),
}

impl Error for HandError {}

impl fmt::Display for HandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFiveCards(n) => write!(f, "Five cards are requied, but {} were given", n),
            Self::NotTwoCards(n) => write!(f, "Two cards are requied, but {} were given", n),
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

    /// Return the first Rank that we see more than once in the given slice of cards.
    ///
    /// Used as a helper for describe function.
    fn first_paired(cards: &[Card]) -> Rank {
        let mut seen = Vec::with_capacity(4);
        for c in cards {
            if seen.contains(&c.rank()) {
                return c.rank();
            }
            seen.push(c.rank());
        }
        unreachable!();
    }

    /// Return the first Rank (other than the given Rank) that we see more than once in the given
    /// slice of cards.
    ///
    /// Used as a helper for describe function.
    fn first_paired_not(cards: &[Card], other: Rank) -> Rank {
        let mut seen = Vec::with_capacity(3);
        for c in cards {
            if c.rank() == other {
                continue;
            } else if seen.contains(&c.rank()) {
                return c.rank();
            }
            seen.push(c.rank());
        }
        unreachable!();
    }

    /// Return the first Rank that we see more than twice in the given slice of cards.
    ///
    /// Used as a helper for describe function.
    fn first_set(cards: &[Card]) -> Rank {
        let mut seen = Vec::with_capacity(3);
        let mut seen_twice = None;
        for c in cards {
            if !seen.contains(&c.rank()) {
                seen.push(c.rank());
            } else if seen_twice.is_none() {
                seen_twice = Some(c.rank());
            } else if seen_twice.unwrap() == c.rank() {
                return c.rank();
            }
        }
        unreachable!();
    }

    /// Return the high card of the straight contained in the given five card slice.
    ///
    /// Used as a helper for describe function
    fn straight_high(c: &[Card]) -> Rank {
        let mut cards: [Card; 5] = [c[0], c[1], c[2], c[3], c[4]];
        cards.sort_unstable();
        cards.reverse();
        match cards[0].rank() {
            Rank::RA => match cards[1].rank() {
                Rank::RK => Rank::RA,
                Rank::R5 => Rank::R5,
                _ => unreachable!(),
            },
            _ => cards[0].rank(),
        }
    }

    /// Return the high card in the given five card slice.
    ///
    /// Used as a helper for describe function
    fn high_card(c: &[Card]) -> Rank {
        let mut cards: [Card; 5] = [c[0], c[1], c[2], c[3], c[4]];
        cards.sort_unstable();
        cards.reverse();
        cards[0].rank()
    }

    pub fn describe(&self) -> String {
        match self.class {
            HandClass::HighCard => format!("{} high", Self::high_card(&self.cards)),
            HandClass::Pair => format!("Pair of {}s", Self::first_paired(&self.cards)),
            HandClass::TwoPair => {
                let first = Self::first_paired(&self.cards);
                let second = Self::first_paired_not(&self.cards, first);
                let mut buf = [first, second];
                buf.sort_unstable();
                buf.reverse();
                format!("Two pair {}s and {}s", buf[0], buf[1])
            }
            HandClass::ThreeOfAKind => {
                format!("Set of {}s", Self::first_set(&self.cards))
            }
            HandClass::Straight => format!("{} high straight", Self::straight_high(&self.cards)),
            HandClass::Flush => format!("{} high flush", Self::high_card(&self.cards)),
            HandClass::FullHouse => {
                let first = Self::first_set(&self.cards);
                let second = Self::first_paired_not(&self.cards, first);
                format!("Boat {}s full of {}s", first, second)
            }
            HandClass::FourOfAKind => {
                format!("Quad {}s", Self::first_paired(&self.cards))
            }
            HandClass::StraightFlush => {
                format!("{} high straight flush", Self::straight_high(&self.cards))
            }
        }
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.beats(other).into()
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Checks all 5-card combinations of the given cards, and returns a Vector of the best
/// 5-card hands. If more than one Hand is returned, they are all equal (`WinState::Tie`).
/// If <5 cards are in the given slice, returns an empty vec.
///
/// This function checks every single possible combination of five cards. Be mindful of this before
/// given it a large number of cards.
///
///    - 6 choose 5: 6
///    - 7 choose 5: 21
///    - 8 choose 5: 56
///    - 10 choose 5: 252
///    - 52 choose 5: 2.6 million
///
/// The original use case was best 5 card hand given 7 cards.
pub fn best_of_cards(cards: &[Card]) -> Vec<Hand> {
    if cards.len() < 5 {
        return vec![];
    }
    let mut hands: Vec<_> = cards
        .iter()
        .combinations(5)
        .map(|combo| {
            // .combinations() gives us a Vec<&Card>, but we want Vec<Card>
            combo.iter().map(|&c| *c).collect::<Vec<Card>>()
        })
        .map(|combo| Hand::new_unchecked(&combo))
        .collect();
    // do r.beats(l) instead of l.beats(r) because we want the first items in the list to be better
    // than the ones that follow. Otherwise we'd have to sort and then reverse afterward.
    hands.sort_unstable_by(|l, r| r.beats(l).into());
    // The best hand is at the front. Return a Vec containing items from the front of the list as
    // long as they tie the best hand.
    let best = hands[0];
    hands
        .into_iter()
        .take_while(|h| h.beats(&best) == WinState::Tie)
        .collect()
}

/// Order all the given hands and return them, best-to-worst.
///
/// Arguments:
///
///   - pockets: Mapping between Account ID and their pocket 2 cards
///   - community: The 5 community cards
///
/// Returns (if no error):
///
///   A Vec where each item is a Vec of (Account ID, Hand) tuples. The outer vec is the ordering
///   (best-to-worst), and inner vecs are so ties can be represented.
///
/// Errors:
///
///   - `HandError::NotTwoCards` if any pocket isn't two cards long
///   - `HandError::NotFiveCards` if the community isn't five cards long
pub fn best_hands<AID: Copy>(
    pockets: &HashMap<AID, [Card; 2]>,
    community: [Card; 5],
) -> Result<Vec<Vec<(AID, Hand)>>, HandError> {
    if pockets.is_empty() {
        // This check is important, as later we pull out the best hand before iterating over the
        // rest.
        return Ok(vec![]);
    }
    if community.len() != 5 {
        return Err(HandError::NotFiveCards(community.len()));
    }
    // Get the best possible 5-card hand for each pocket
    let mut hands = vec![];
    for (account_id, pocket) in pockets {
        if pocket.len() != 2 {
            return Err(HandError::NotTwoCards(pocket.len()));
        }
        let mut cards = Vec::with_capacity(7);
        cards.extend_from_slice(pocket);
        cards.extend_from_slice(&community);
        assert_eq!(cards.len(), 7);
        let hand = best_of_cards(&cards)[0];
        hands.push((account_id, hand));
    }
    // Do left beats right, as in this function we want the best to be at the end of the list,
    // which is the opposite of what we often do in other functions.
    hands.sort_by(|l, r| l.1.beats(&r.1).into());
    // We have all hands sorted now. It is time to coalesce ties together by wrapping all hands
    // with a vec of length one and tie-ing hands together into a vec of length >1
    let mut ret: Vec<Vec<(AID, Hand)>> = vec![];
    let mut inner: Vec<(AID, Hand)> = vec![];
    let mut current_best = hands[hands.len() - 1].1;
    while let Some((account_id, hand)) = hands.pop() {
        match hand.cmp(&current_best) {
            Ordering::Equal => {
                inner.push((*account_id, hand));
            }
            Ordering::Less => {
                ret.push(inner.clone());
                inner.truncate(0);
                inner.push((*account_id, hand));
                current_best = hand;
            }
            Ordering::Greater => {
                unreachable!();
            }
        };
    }
    if !inner.is_empty() {
        ret.push(inner);
    }
    Ok(ret)
}

#[cfg(test)]
mod test_best_of_cards {
    use super::*;
    use crate::deck::*;

    fn one_best(s: &'static str, hc: HandClass, high_card: Card) {
        let hands = best_of_cards(&cards_from_str(s));
        for hand in &hands {
            println!("{}", hand);
        }
        assert_eq!(hands.len(), 1);
        let hand = hands[0];
        assert_eq!(hand.class, hc);
        let card = hand.cards.iter().max().unwrap();
        assert_eq!(card.rank(), high_card.rank());
        assert_eq!(card.suit(), high_card.suit());
    }

    fn multi_best(s: &'static str, hc: HandClass, n: usize) {
        let hands = best_of_cards(&cards_from_str(s));
        for hand in &hands {
            println!("{}", hand);
        }
        assert_eq!(hands.len(), n);
        assert_eq!(hands[0].class, hc);
    }

    #[test]
    fn multiple_straights() {
        one_best("Ac2d3h4s5c6dTh", HandClass::Straight, ['6', DIAMOND].into());
    }

    #[test]
    fn multiple_straights_tie() {
        multi_best("Kc2d3h4s5c6d6h", HandClass::Straight, 2);
        multi_best("2d3h4s5c6d6h6s", HandClass::Straight, 3);
    }

    #[test]
    fn straight_vs_flush() {
        one_best("Th9s8h7h6h5h2c", HandClass::Flush, ['T', HEART].into());
    }
}

#[cfg(test)]
mod test_best_hands {
    use super::*;

    #[test]
    fn basic() {
        let mut map: HashMap<i32, [Card; 2]> = HashMap::new();
        map.insert(1, [['A', 'c'].into(), ['A', 'd'].into()]);
        map.insert(2, [['A', 'h'].into(), ['A', 's'].into()]);
        map.insert(3, [['K', 'h'].into(), ['K', 's'].into()]);
        let comm = [
            ['2', 'c'].into(),
            ['3', 'd'].into(),
            ['5', 'h'].into(),
            ['9', 's'].into(),
            ['T', 'c'].into(),
        ];
        let ret = best_hands(&map, comm).unwrap();
        for (idx, inner) in ret.iter().enumerate() {
            println!("{}:", idx);
            for h in inner {
                println!("    {} {}", h.0, h.1);
            }
        }
        assert_eq!(ret.len(), 2);
        assert_eq!(ret[0].len(), 2);
        assert_eq!(ret[1].len(), 1);
        assert_eq!(ret[0][0].1.class, HandClass::Pair);
        assert_eq!(ret[0][0].1.cards[0].rank(), Rank::RA);
        assert_eq!(ret[1][0].1.class, HandClass::Pair);
        assert_eq!(ret[1][0].1.cards[0].rank(), Rank::RK);
    }
}

#[cfg(test)]
mod test_hand {
    use super::*;
    use crate::deck::cards_from_str;
    use crate::deck::Deck;
    use std::iter;

    #[test]
    fn wrong_sizes() {
        let mut deck = Deck::default();
        for n in [0, 1, 2, 3, 4, 6, 7] {
            let cards: Vec<Card> = iter::repeat_with(|| deck.draw().unwrap()).take(n).collect();
            let hand = Hand::new(&cards);
            assert!(hand.is_err());
        }
    }

    #[test]
    fn correct_size() {
        let mut deck = Deck::default();
        let cards: Vec<Card> = iter::repeat_with(|| deck.draw().unwrap()).take(5).collect();
        let hand = Hand::new(&cards);
        assert!(hand.is_ok());
    }

    /// Verify that the first hand is greater than (wins compared to) the second hand. Also verify
    /// the other equality properties that would also be true.
    fn beats_helper1(s1: &'static str, s2: &'static str) {
        let h1 = Hand::new_unchecked(&cards_from_str(s1));
        let h2 = Hand::new_unchecked(&cards_from_str(s2));
        assert!(h1 > h2);
        assert!(h2 < h1);
        assert_eq!(h1, h1.clone());
        assert_eq!(h2, h2.clone());
        // same as above, but without Ord/PartialOrd wrapper
        assert_eq!(h1.beats(&h2), WinState::Win);
        assert_eq!(h2.beats(&h1), WinState::Lose);
        assert_eq!(h1.beats(&h1.clone()), WinState::Tie);
        assert_eq!(h2.beats(&h2.clone()), WinState::Tie);
    }

    #[test]
    fn beats() {
        for (s1, s2) in [("AsKsQsJsTs", "KdQdJdTd9d"), ("AsKsQsJsTs", "Td8s6d4d2d")] {
            beats_helper1(s1, s2);
        }
    }
}

#[cfg(test)]
mod test_hand_describe {
    use super::*;
    use crate::deck::cards_from_str;

    fn is(hand: &'static str, desc: &'static str) {
        assert_eq!(Hand::new_unchecked(&cards_from_str(hand)).describe(), desc);
    }

    #[test]
    fn high_card() {
        is("Ah6h5d4c3s", "A high");
        is("6hAh5d4c3s", "A high");
        is("7c5d4h3s2s", "7 high");
    }

    #[test]
    fn pair() {
        is("AcKdQh6s6c", "Pair of 6s");
        is("Ac6s6cKdQh", "Pair of 6s");
        is("AcAs6cKdQh", "Pair of As");
    }

    #[test]
    fn two_pair() {
        is("AcAdKcKd4d", "Two pair As and Ks");
        is("4dKcKdAcAd", "Two pair As and Ks");
        is("6c2c4s6d2d", "Two pair 6s and 2s");
    }

    #[test]
    fn set() {
        is("AcAdAhKcQc", "Set of As");
        is("TcKdThTsQc", "Set of Ts");
    }

    #[test]
    fn straight() {
        is("AdKsQsJsTs", "A high straight");
        is("KdAsTsJsQs", "A high straight");
        is("Ad2s4s3s5s", "5 high straight");
        is("8d4s6s5s7s", "8 high straight");
    }

    #[test]
    fn flush() {
        is("Ac8c7c6c5c", "A high flush");
        is("Tc8c7c6c5c", "T high flush");
        is("8cTc5c6c6c", "T high flush");
        is("7c6c5c4c2c", "7 high flush");
    }

    #[test]
    fn full_house() {
        is("AcKcAdKdAs", "Boat As full of Ks");
        is("2cKc2dKd2s", "Boat 2s full of Ks");
    }

    #[test]
    fn quads() {
        is("AcAdAhAsKc", "Quad As");
        is("2c2d2h2s3c", "Quad 2s");
    }

    #[test]
    fn straight_flush() {
        is("AsKsQsJsTs", "A high straight flush");
        is("KsAsTsJsQs", "A high straight flush");
        is("As2s4s3s5s", "5 high straight flush");
        is("8s4s6s5s7s", "8 high straight flush");
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
}

#[cfg(test)]
mod test_hand_class_beats {
    use super::*;
    use crate::deck::cards_from_str;

    fn win_lose(s1: &'static str, s2: &'static str, hc: HandClass) {
        let h1 = Hand::new_unchecked(&cards_from_str(s1));
        let h2 = Hand::new_unchecked(&cards_from_str(s2));
        assert_eq!(h1.class, hc);
        assert_eq!(h2.class, hc);
        println!("win? {} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Win);
        println!("lose? {} vs {}", h2, h1);
        assert_eq!(h2.beats(&h1), WinState::Lose);
    }

    fn tie(s1: &'static str, s2: &'static str, hc: HandClass) {
        let h1 = Hand::new_unchecked(&cards_from_str(s1));
        let h2 = Hand::new_unchecked(&cards_from_str(s2));
        assert_eq!(h1.class, hc);
        assert_eq!(h2.class, hc);
        println!("tie? {} vs {}", h1, h2);
        assert_eq!(h1.beats(&h2), WinState::Tie);
    }

    #[test]
    fn straight_flush_tie() {
        for (s1, s2) in [
            ("AcKcQcJcTc", "AdKdQdJdTd"),
            ("KcQcJcTc9c", "KdQdJdTd9d"),
            ("5c4c3c2cAc", "5d4d3d2dAd"),
        ] {
            tie(s1, s2, HandClass::StraightFlush);
        }
    }

    #[test]
    fn straight_flush() {
        for (s1, s2) in [
            ("AcKcQcJcTc", "KdQdJdTd9d"),
            ("6c5c4c3c2c", "5d4d3d2dAd"),
            ("AcKcQcJcTc", "5d4d3d2dAd"),
        ] {
            win_lose(s1, s2, HandClass::StraightFlush);
        }
    }

    #[test]
    fn quads_tie() {
        // this should be impossible in typical single deck poker, but check for it anyway since
        // the logic doesn't care
        for (s1, s2) in [("2c2d2h2s3c", "2c2d2h2s3d")] {
            tie(s1, s2, HandClass::FourOfAKind);
        }
    }

    #[test]
    fn quads() {
        for (s1, s2) in [("4c4d4h4s3c", "3c3d3h3s2d"), ("4c4d4h4s5c", "4c4d4h4s3c")] {
            win_lose(s1, s2, HandClass::FourOfAKind);
        }
    }

    #[test]
    fn full_house_tie() {
        for (s1, s2) in [("AcAdAhKcKd", "AdAhAsKhKs")] {
            tie(s1, s2, HandClass::FullHouse);
        }
    }

    #[test]
    fn full_house() {
        for (s1, s2) in [("4c4d4h3s3c", "3c3d3h2s2d"), ("4c4d4h5s5c", "4c4d4h3s3c")] {
            win_lose(s1, s2, HandClass::FullHouse);
        }
    }

    #[test]
    fn flush_tie() {
        for (s1, s2) in [("AsKsQsJs2s", "AdKdQdJd2d")] {
            tie(s1, s2, HandClass::Flush);
        }
    }

    #[test]
    fn flush() {
        for (s1, s2) in [("AsKsQsJs3s", "AdKdQdJd2d"), ("As6s5s4s3s", "Kd7d6d5d4d")] {
            win_lose(s1, s2, HandClass::Flush);
        }
    }

    #[test]
    fn straight_tie() {
        for (s1, s2) in [("AsKsQsJsTd", "AcKcQcJcTs")] {
            tie(s1, s2, HandClass::Straight);
        }
    }

    #[test]
    fn straight() {
        for (s1, s2) in [
            ("AsKsQsJsTd", "KcQcJcTc9s"),
            ("AsKsQsJsTd", "Ac2c3c4c5s"),
            ("6s5s4s3s2d", "Ac2c3c4c5s"),
        ] {
            win_lose(s1, s2, HandClass::Straight);
        }
    }

    #[test]
    fn set_tie() {
        for (s1, s2) in [("AcAdAh4s3d", "AsAcAd4c3s"), ("3c3d3hAsKd", "3s3c3dAcKs")] {
            tie(s1, s2, HandClass::ThreeOfAKind);
        }
    }

    #[test]
    fn set() {
        for (s1, s2) in [
            ("AcAdAh4s3d", "AsAcAd3c2s"),
            ("9c9d9hTsJd", "9s9c9d2c3s"),
            ("9c9d9h6s3d", "9s9c9d3c2s"),
        ] {
            win_lose(s1, s2, HandClass::ThreeOfAKind);
        }
    }

    #[test]
    fn two_pair_tie() {
        for (s1, s2) in [("AsAsKsKsTd", "AcAcKcKcTs")] {
            tie(s1, s2, HandClass::TwoPair);
        }
    }

    #[test]
    fn two_pair() {
        for (s1, s2) in [("AsAsKsKsJd", "AcAcKcKcTs"), ("AsAsKsKsJd", "AcAcQcQcKs")] {
            win_lose(s1, s2, HandClass::TwoPair);
        }
    }

    #[test]
    fn pair_tie() {
        for (s1, s2) in [("AcAd5h4s3d", "AcAd5s4c3h"), ("2c2d5h4s3d", "2c2d5s4c3h")] {
            tie(s1, s2, HandClass::Pair);
        }
    }

    #[test]
    fn pair() {
        for (s1, s2) in [
            ("AcAdKh4s3d", "AcAd5h4s3d"),
            ("AcAd5h4s3d", "AcAd5h4s2d"),
            ("2c2d6h4s3d", "2c2d5h4s3d"),
        ] {
            win_lose(s1, s2, HandClass::Pair);
        }
    }

    #[test]
    fn high_card_tie() {
        for (s1, s2) in [("KcQdJhTs5c", "KdQhJsTc5d")] {
            tie(s1, s2, HandClass::HighCard);
        }
    }

    #[test]
    fn high_card() {
        for (s1, s2) in [
            ("Ac7d6h5s4d", "Ac6d5h4s3d"),
            ("AcKdQhJs7d", "AcKdQhJs3d"),
            ("8c7d6h4s3d", "7c6d5h3s2d"),
        ] {
            win_lose(s1, s2, HandClass::HighCard);
        }
    }
}
