use super::card::*;
use enum_map::EnumMap;
use itertools::Itertools;
use std::str::FromStr;

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
    RoyalFlush,
}

const ALL_HAND_CLASSES: [HandClass; 10] = [
    HandClass::RoyalFlush,
    HandClass::StraightFlush,
    HandClass::FourOfAKind,
    HandClass::FullHouse,
    HandClass::Flush,
    HandClass::Straight,
    HandClass::ThreeOfAKind,
    HandClass::TwoPair,
    HandClass::Pair,
    HandClass::HighCard,
];

const LOW_RANK_STRAIGHT: [Rank; 5] = [Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five];

#[derive(Copy, Clone, Debug)]
pub struct FinalHandResult {
    pub cards: [Card; 5],
    pub class: HandClass,
}

impl PartialOrd for FinalHandResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FinalHandResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        // Easy case first
        if self.class > other.class {
            return Ordering::Greater;
        } else if self.class < other.class {
            return Ordering::Less;
        } else if self.eq(other) {
            return Ordering::Equal;
        } else {
            for i in 0..5 {
                // Sorted, so left to right rank comarisons should be ordered
                if self.cards[i] > other.cards[i] {
                    return Ordering::Greater;
                } else if self.cards[i] < other.cards[i] {
                    return Ordering::Less;
                } else {
                    continue;
                }
            }
            unreachable!();
        }
    }
}

impl Eq for FinalHandResult {}

impl PartialEq for FinalHandResult {
    fn eq(&self, other: &Self) -> bool {
        if self.class == other.class {
            return self.cards.iter().map(|x| x.rank).counts()
                == other.cards.iter().map(|x| x.rank).counts();
        }
        false
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Hand {
    pub pocket: Option<[Card; 2]>,
    pub board: [Option<Card>; 5],
}

/// Given some known cards and the number of unknown cards yet to come,
/// determine whether or not it is possible to make a straight.
fn is_straight_possible(h: impl Iterator<Item = Card> + Clone, cards_to_come: u8) -> bool {
    // It doesn't matter what we have (we could have nothing!) if 5 cards are
    // yet to come. Those 5 could be in order and make a straight.
    if cards_to_come >= 4 {
        return true;
    }
    // We only care about ranks here. So let's operate on those.
    // Further, we only care about unique ranks. So do that.
    // Resort because in tests cards are not always sorted exactly correctly.
    // (AKQ...432)
    let mut ranks: Vec<i8> = h.map(|c| c.rank.into()).unique().sorted().rev().collect();
    // If there's an Ace, it'll be first. And it can be either high or low in a
    // straight. So to make the rest of this algo easier, add an Ace to the end,
    // valued as 1 instead of 14.
    if ranks.is_empty() {
        return false;
    } else if ranks[0] == Rank::Ace.into() {
        // Yes I really am doing 2-1 = 1 here, because fuck you maybe in the
        // future Rank::Two isn't 2.
        //
        // Defensive programming, bitches. Learn it. Do it.
        let two: i8 = Rank::Two.into();
        ranks.push(two - 1);
    }
    // We will check a first card (the higher) and a second card (WILL be
    // lower). The index of the second is this many more than the index of the
    // first.
    // The logic for this (and the key insight for the rest of this function):
    //
    // Assume the rank list only contains unique ranks and they are sorted. In
    // order for a straight to be possible, you need to be able to find a pair
    // of cards that is close enough together s.t. other (known) cards in the
    // hand and the (unknown) cards yet to come can fill in between those two
    // chosen cards and constitute a straight. Call the chosen cards 'A' and
    // 'B'. When the list is ordered and contains unique elements, we can
    // manipulate the number of elements between A and B s.t. the number of
    // known cards is implied and the gaps the unknown cards to come (CTC)
    // must fill is trivially calculateable. The difference between a valid A
    // and B that can make a straight given these assumptions is <= 4.
    //
    // Given ranks K986 and 2 CTC, we need to find an A and B s.t. A, B, and
    // the 1 card between them (3 known cards) plus the 2 CTC (2 unkown cards)
    // could all together constitute a straight. K (13) minus 8 is 5, too many.
    // Move on. 9-6 <= 4, which works. The 8 between them we known, and the
    // remaining unknown cards can either be T7 or 75 (T9876 or 98765).
    //
    // Given ranks A95 and 3 CTC, we would do A (14) minus 9, get 5, too many,
    // move on. 9-5<=4, which works for 98765
    //
    // Given ranks 987 and 3 CTC, we would do 9-8 <= 4, which works for many
    // straights involving 98. (Not to mention the straights we'd get with 87).
    //
    // Given ranks T987 and 1 CTC, we do 10-7 <= 4. (JT987 and T9876) yes
    // Ranks T986 and 1 CTC: 10-6 <= 4 (T9876) yes
    // Ranks T876 and 1 CTC: 10-6 <= 4 (T9876) yes
    // Ranks T765 and 1 CTC: 10-5 is 5 so no straight possible. The 765 are
    // connected, but with 1 CTC we can't possibly make a straight.
    let gap_size = match cards_to_come {
        0 => 4,   // 3 known cards between
        1 => 3,   // 2 known cards between
        2 => 2,   // 1 known cards between
        3.. => 1, // 0 known cards between
    };
    // Iterate down the list once, calculating the diff between A and B. As
    // described at length above, it must be 4 or less if a straight is
    // possible. This is true no matter how many CTC between we manipulated the
    // number of cards between A and B in this ordered list of unique ranks
    // based on the number of CTC.
    let mut first = 0;
    let mut last = first + gap_size;
    while last < ranks.len() {
        let diff = ranks[first] - ranks[last];
        if diff <= 4 {
            return true;
        }
        first += 1;
        last += 1;
    }
    false
}

// Helper function to take an iterator of cards and determine if there is a straight
// The cards MUST be sorted power to the left, Ace at [0]
fn cards_have_straight(h: impl Iterator<Item = Card> + Clone) -> Option<[Option<Card>; 5]> {
    if h.clone().count() >= 5 {
        let mut in_a_row = 0usize;
        let mut prev: Rank = Rank::Two;
        let mut cards: [Option<Card>; 5] = [None; 5];
        for c in h.clone() {
            // Left to right is done with full straight
            if in_a_row == 5 {
                return Some(cards);
            }
            // Cards are sorted high to low be default
            if in_a_row == 0 {
                cards[in_a_row] = Some(c);
                prev = c.rank;
                in_a_row += 1;
            } else if c.rank.value() == prev.value() - 1 {
                cards[in_a_row] = Some(c);
                prev = c.rank;
                in_a_row += 1;
            } else if c.rank == prev {
                continue;
            } else {
                prev = Rank::Two;
                in_a_row = 0;
                // We don't need to re-null cards because it only returns if iar gets back to 5
                // Meaning it has to re-write the entire array anyway.
            }
        }
        if in_a_row == 5 {
            return Some(cards);
        }
        let mut flag = true;
        let v: Vec<Rank> = h.clone().map(|c| c.rank).collect();
        for r in LOW_RANK_STRAIGHT {
            if !v.contains(&r) {
                flag = false;
                break;
            }
        }
        if flag {
            for (i, r) in LOW_RANK_STRAIGHT.iter().enumerate() {
                cards[i] = h.clone().find(|x| x.rank == *r);
            }
            // Force Ace to the right of low rank sort to fix hand comparisons
            let t = cards[0];
            cards[0] = cards[4];
            cards[4] = t;
            return Some(cards);
        }
        // Handle low straight
    }
    None
}

impl FromStr for Hand {
    // TODO
    type Err = String;

    fn from_str(s: &str) -> Result<Hand, Self::Err> {
        let mut i = s.chars();
        let mut cards: [Option<Card>; 5] = [None; 5];
        let mut ci = 0usize;
        let p0 = Card::from([
            i.nth(0).ok_or(String::from("Failed to parse hand"))?,
            i.nth(0).ok_or(String::from("Failed to parse hand"))?,
        ]);
        let p1 = Card::from([
            i.nth(0).ok_or(String::from("Failed to parse hand"))?,
            i.nth(0).ok_or(String::from("Failed to parse hand"))?,
        ]);
        let pocket = Some([p0, p1]);
        for mut s in &i.chunks(2) {
            let c = Card::from([
                s.nth(0).ok_or(String::from("Failed to parse hand"))?,
                s.nth(0).ok_or(String::from("Failed to parse hand"))?,
            ]);
            cards[ci] = Some(c);
            ci += 1;
        }
        // Make sure there are no duplicates
        let hand = Hand::new_with_pocket(pocket, cards);
        if hand.get_hand_iter().unique().count() != hand.card_count() {
            return Err(String::from("Found duplicate cards"));
        }

        Ok(hand)
    }
}

impl Hand {
    pub fn new_without_pocket(board: [Option<Card>; 5]) -> Self {
        Hand {
            pocket: None,
            board,
        }
    }

    pub fn new_with_pocket(pocket: Option<[Card; 2]>, board: [Option<Card>; 5]) -> Self {
        Hand { pocket, board }
    }

    pub fn from_iter(cards: impl IntoIterator<Item = Card> + Clone) -> Self {
        let mut cards = cards.clone().into_iter();
        let p0 = cards.nth(0).expect("from_iter empty");
        let p1 = cards.nth(0).expect("from_iter with 1 card");
        let pocket = Some([p0, p1]);
        let mut board = [None; 5];
        let mut bi = 0usize;
        for c in cards {
            board[bi] = Some(c);
            bi += 1;
        }
        Hand { pocket, board }
    }

    /// Clones the current Hand, dropping the Pocket
    pub fn board_only(&self) -> Hand {
        Hand::new_without_pocket(self.board.clone())
    }

    pub fn get_hand(&self) -> [Option<Card>; 7] {
        let mut a: [Option<Card>; 7] = [None; 7];

        if self.pocket.is_some() {
            for (i, c) in self.pocket.as_ref().unwrap().iter().enumerate() {
                a[i] = Some(*c);
            }
        }
        for (i, c) in self.board.iter().enumerate() {
            a[i + 2] = *c;
        }

        a
    }

    /// Helper function for how many cards are currently dealt
    fn card_count(&self) -> usize {
        self.get_hand_iter().count()
    }

    /// Helper function for how many cards remain unseen
    fn cards_left(&self) -> usize {
        7 - self.card_count()
    }

    /// Helper function for how many suits are currently represented
    fn suit_count(&self) -> usize {
        self.get_hand_iter().map(|x| x.suit).unique().count()
    }

    /// helper function for how many ranks are currently represented
    fn rank_count(&self) -> usize {
        self.get_hand_iter().map(|x| x.rank).unique().count()
    }

    /// helper function for getting hash_map of ranks
    fn ranks(&self) -> EnumMap<Rank, usize> {
        let mut em: EnumMap<Rank, usize> = EnumMap::from_array([0usize; 13]);
        for c in self.get_hand_iter() {
            em[c.rank] += 1;
        }
        em
    }
    /// helper function for getting hash_map of ranks
    fn suits(&self) -> EnumMap<Suit, usize> {
        let mut em: EnumMap<Suit, usize> = EnumMap::from_array([0usize; 4]);
        for c in self.get_hand_iter() {
            em[c.suit] += 1;
        }
        em
    }

    // helper function to get all cards of a specific suit
    fn get_cards_by_suit_iter(&self, s: Suit) -> impl Iterator<Item = Card> + Clone {
        self.get_hand_iter().filter(move |x| x.suit == s)
    }

    // helper function to get all cards of a specific suit
    fn get_cards_by_rank_iter(&self, r: Rank) -> impl Iterator<Item = Card> {
        self.get_hand_iter().filter(move |x| x.rank == r)
    }
    /// helper function that returns an iterator over pairs
    /// Since the logic is basically the same it also handles trips
    fn pairs(&self, count: usize) -> impl Iterator<Item = Card> + Clone {
        let rhm = self.ranks();
        let mut sp = Vec::new();
        for (k, v) in rhm.into_iter() {
            if v == count {
                sp.push(k);
            }
        }
        // Returns at most two pairs or one trips
        // 2 == 4 because take() will exit early if only one pair was found
        let r = if count == 2 || count == 4 { 4 } else { 3 };
        self.get_hand_iter()
            .filter(move |&x| sp.contains(&x.rank))
            .take(r)
    }

    /// Gets the cards currently available to the hand.
    /// Cards will be sorted in the order of highest rank at index 0
    /// Makes finding the highest kicker easier
    pub fn get_hand_iter(&self) -> impl Iterator<Item = Card> + Clone {
        self.get_hand()
            .into_iter()
            .filter_map(|x| x)
            .sorted_unstable()
            .rev()
    }

    /// Helper function to get a sorted hand iterator minus cards in a parameter
    fn get_filtered_hand_iter(
        &self,
        f: impl Iterator<Item = Card> + Clone,
    ) -> impl Iterator<Item = Card> + Clone {
        // Have to evaluate and re-return to break the &mut requirement of the lazy iterator
        let mut v: Vec<Card> = self
            .get_hand_iter()
            .filter(|&x| !f.clone().contains(&x))
            .collect();
        v.sort_unstable();
        v.into_iter()
    }

    /// Helper function to fill kickers
    fn fill_kickers(&self, c: [Option<Card>; 5], index: usize) -> [Option<Card>; 5] {
        let mut c = c;
        let mut remaining = self.get_filtered_hand_iter(c.into_iter().filter_map(|x| x));
        for i in index..5 {
            c[i] = remaining.nth(0);
        }
        c
    }

    /// Helper function to indicate if the best hand is just the board
    pub fn playing_board(&self) -> bool {
        self.get_best_possible_hand_result() == self.board_only().get_best_possible_hand_result()
    }

    fn test_result(&self, hr: HandClass) -> HaveResult {
        use HandClass::*;
        let tfn = match hr {
            HighCard => <Self as HandSolver>::high_card,
            Pair => <Self as HandSolver>::pair,
            TwoPair => <Self as HandSolver>::two_pair,
            ThreeOfAKind => <Self as HandSolver>::three_kind,
            Straight => <Self as HandSolver>::straight,
            Flush => <Self as HandSolver>::flush,
            FullHouse => <Self as HandSolver>::full_house,
            FourOfAKind => <Self as HandSolver>::four_kind,
            StraightFlush => <Self as HandSolver>::straight_flush,
            RoyalFlush => <Self as HandSolver>::royal_flush,
        };
        tfn(&self)
    }
    pub fn get_best_possible_hand_result(&self) -> HandClass {
        // Hacky fix for hands with only pocket cards
        if self.card_count() == 2 {
            return HandClass::RoyalFlush;
        }
        for hr in ALL_HAND_CLASSES {
            if self.test_result(hr).bool() {
                return hr;
            }
        }
        // Rust doesn't understand that can_have_high_card always returns true
        unreachable!("Best possible hand failed")
    }

    pub fn get_current_hand_class(&self) -> HandClass {
        for r in ALL_HAND_CLASSES.iter() {
            match self.test_result(*r) {
                HaveResult::Has(_x) => {
                    return *r;
                }
                _ => {
                    continue;
                }
            }
        }
        unreachable!("Current hand class failed")
    }

    pub fn get_current_best_hand(&self) -> (HandClass, [Option<Card>; 5]) {
        for r in ALL_HAND_CLASSES.iter() {
            match self.test_result(*r) {
                HaveResult::Has(x) => {
                    return (*r, x);
                }
                _ => {
                    continue;
                }
            }
        }
        unreachable!("Current best hand failed")
    }

    pub fn finalize_hand(self) -> FinalHandResult {
        assert!(self.card_count() >= 5);
        // Default, probably want to unsafe this later
        let mut cards: [Card; 5] = [Card::from_str("Ah").unwrap(); 5];
        let (class, c) = self.get_current_best_hand();
        for (ci, c) in c.into_iter().enumerate() {
            cards[ci] = c.unwrap();
        }
        FinalHandResult { cards, class }
    }
}

pub enum HaveResult {
    CantHave,
    CanHave,
    Has([Option<Card>; 5]),
}

impl HaveResult {
    /// Helper function that combines Has and CanHav
    pub fn bool(&self) -> bool {
        match self {
            HaveResult::CantHave => false,
            _ => true,
        }
    }
}

impl HandSolver for Hand {
    fn royal_flush(&self) -> HaveResult {
        // Has
        if self.suits().values().max().unwrap() >= &5 {
            let mut sm = Suit::Heart;
            for (s, i) in self.suits() {
                if i >= 5 {
                    sm = s;
                }
            }
            let flush_iter = self.get_cards_by_suit_iter(sm);
            let has_sf = cards_have_straight(flush_iter);
            if has_sf.is_some() {
                // Cards are sorted with ace to left, so high straight is always cards[0] = ace and cards[4] = ten
                // This is only true when ranks are deduped, or, as in this case, we are only checking one suit
                let cards = has_sf.unwrap();
                if cards[0].as_ref().unwrap().rank == Rank::Ace
                    && cards[4].as_ref().unwrap().rank == Rank::Ten
                {
                    return HaveResult::Has(cards);
                }
            }
        }

        // Can Have
        //TODO

        // Can't Have
        HaveResult::CantHave
    }

    fn straight_flush(&self) -> HaveResult {
        // Has
        // Checking flush is easy
        let mut sm = Suit::Heart;
        if self.suits().values().max().unwrap() >= &5 {
            for (s, i) in self.suits() {
                if i >= 5 {
                    sm = s;
                }
            }
            let flush_iter = self.get_cards_by_suit_iter(sm);
            let has_sf = cards_have_straight(flush_iter);
            if has_sf.is_some() {
                return HaveResult::Has(has_sf.unwrap());
            }
        }

        // Can Have
        for suit in ALL_SUITS {
            if is_straight_possible(self.get_cards_by_suit_iter(suit), self.cards_left() as u8) {
                return HaveResult::CanHave;
            }
        }

        // Can't Have
        HaveResult::CantHave
    }

    fn four_kind(&self) -> HaveResult {
        let mut cards: [Option<Card>; 5] = [None; 5];
        let mut ci = 0usize;
        // Has
        if self.pairs(4).count() == 4 {
            for c in self.pairs(4) {
                cards[ci] = Some(c);
                ci += 1;
            }
            return HaveResult::Has(self.fill_kickers(cards, ci));
        }

        // Can Have
        if self.ranks().values().max().unwrap() + self.cards_left() >= 4 {
            return HaveResult::CanHave;
        }

        // Can't Have
        HaveResult::CantHave
    }

    fn full_house(&self) -> HaveResult {
        // Has
        let p = self.pairs(2);
        let pc = p.clone().count();
        if self.card_count() >= 5 {
            let t = self.pairs(3);
            let tc = t.clone().count();
            if pc >= 2usize && tc == 3usize {
                let mut cards: [Option<Card>; 5] = [None; 5];
                let mut ci = 0usize;
                for c in p {
                    cards[ci] = Some(c);
                    ci += 1;
                }
                for c in t {
                    cards[ci] = Some(c);
                    ci += 1;
                }
                return HaveResult::Has(cards);
            }
        }

        // Can Have
        // Have four cards left
        if self.cards_left() >= 4 {
            return HaveResult::CanHave;
        }

        let max_ranks = *self.ranks().values().max().unwrap();

        // Has trips and 1 card left
        // Not actually possible to test in current framework. Trips + 1 best
        // Is always quads.
        if max_ranks >= 3 && self.cards_left() >= 1 {
            return HaveResult::CanHave;
        }
        // Has two pair and one card left
        if pc >= 4 && self.cards_left() >= 1 {
            return HaveResult::CanHave;
        }

        // Has pair and two cards left
        if max_ranks >= 2 && self.cards_left() >= 2 {
            return HaveResult::CanHave;
        }

        // Can't Have
        HaveResult::CantHave
    }

    fn flush(&self) -> HaveResult {
        // Has
        if self.suits().values().max().unwrap() >= &5 {
            let mut sm = Suit::Heart;
            for (s, i) in self.suits() {
                if i >= 5 {
                    sm = s;
                }
            }
            let mut cards = [None; 5];
            for (i, c) in self.get_cards_by_suit_iter(sm).enumerate() {
                if i >= 5 {
                    break;
                }
                cards[i] = Some(c);
            }
            return HaveResult::Has(cards);
        }

        // Can Have
        if self.cards_left() + self.suits().values().max().unwrap() >= 5 {
            return HaveResult::CanHave;
        }

        // Can't Have
        HaveResult::CantHave
    }

    fn straight(&self) -> HaveResult {
        // Has
        if self.card_count() >= 5 {
            let has_s = cards_have_straight(self.get_hand_iter());
            if let Some(s) = has_s {
                return HaveResult::Has(s);
            }
        }

        // Can Have9
        if is_straight_possible(self.get_hand_iter(), self.cards_left() as u8) {
            return HaveResult::CanHave;
        }

        // Can't Have
        HaveResult::CantHave
    }

    fn two_pair(&self) -> HaveResult {
        // Has
        let mut ci = 0usize;
        let mut ca: [Option<Card>; 5] = [None; 5];
        for c in self.pairs(2) {
            ca[ci] = Some(c);
            ci += 1;

            if ci >= 4 {
                return HaveResult::Has(self.fill_kickers(ca, ci));
            }
        }

        // Can Have
        if (ci >= 2 && self.cards_left() >= 1) || self.cards_left() >= 3 {
            return HaveResult::CanHave;
        }

        // Can't Have
        HaveResult::CantHave
    }

    fn pair(&self) -> HaveResult {
        // Has
        for _c in self.pairs(2) {
            let mut ca: [Option<Card>; 5] = [None; 5];
            let mut ci = 0usize;
            let pairs = self.pairs(2);
            for c in pairs {
                ca[ci] = Some(c);
                ci += 1;
            }
            return HaveResult::Has(self.fill_kickers(ca, ci));
        }

        // Can Have
        if self.cards_left() >= 1 {
            HaveResult::CanHave
        } else {
            HaveResult::CantHave
        }
    }

    fn three_kind(&self) -> HaveResult {
        // Has Trips
        let mut ca: [Option<Card>; 5] = [None; 5];
        let mut ci = 0usize;
        let pairs = self.pairs(3);
        for c in pairs {
            ca[ci] = Some(c);
            ci += 1;
        }
        if ci > 0 {
            return HaveResult::Has(self.fill_kickers(ca, ci));
        }

        // Check if it's possible
        // If there are at least two cards left to go or 1 + pair
        let has_pair = self.pairs(2).nth(0).is_some();
        if self.cards_left() >= 2 || (self.cards_left() >= 1 && has_pair) {
            return HaveResult::CanHave;
        }
        HaveResult::CantHave
    }

    fn high_card(&self) -> HaveResult {
        let ca: [Option<Card>; 5] = [None; 5];
        HaveResult::Has(self.fill_kickers(ca, 0))
    }
}

/// This trait contains methods of looking up whether a given iterator (any number of cards) has, or could have,
/// a HandResult
/// have_* functions imply can_have_*, but the inverse is not true.
/// # Important
/// have_* functions do NOT return the best hand, only the best hand for that category.
/// for example, `have_pair` only attempts to return 5 cards that contains the strongest pair and the strongest kickers
/// i.e., in the hand AAJ333 `have_pair` would return the best hand as AAJ33
/// As such, have_* functions should be called in order of power when trying to find the best hand.
pub trait HandSolver {
    fn royal_flush(&self) -> HaveResult;
    fn straight_flush(&self) -> HaveResult;
    fn four_kind(&self) -> HaveResult;
    fn full_house(&self) -> HaveResult;
    fn flush(&self) -> HaveResult;
    fn straight(&self) -> HaveResult;
    fn three_kind(&self) -> HaveResult;
    fn two_pair(&self) -> HaveResult;
    fn pair(&self) -> HaveResult;
    fn high_card(&self) -> HaveResult;
}

#[cfg(test)]
mod test_class {
    use super::*;

    fn best_partial_hand_class(s: &'static str) -> HandClass {
        let h = Hand::from_str(s).unwrap();
        //dbg!(&h);
        h.get_current_hand_class()
    }

    fn get_card_array(s: &'static str) -> Vec<Card> {
        let mut v: Vec<Card> = Vec::new();
        for c in &s.chars().chunks(2) {
            let ss: Vec<char> = c.take(2).collect();
            let ss: [char; 2] = <[char; 2]>::try_from(ss).unwrap();
            v.push(Card::from(ss));
        }
        v.sort();
        v.reverse();
        v
    }

    #[test]
    fn test_cards_straight() {
        let c = get_card_array("2h3d4c5s6h");
        dbg!(&c);
        let sc = cards_have_straight(c.into_iter());
        assert!(sc.is_some());
    }

    #[test]
    fn high_card_class() {
        assert_eq!(best_partial_hand_class("Ah4s"), HandClass::HighCard);
        assert_eq!(best_partial_hand_class("5h4s"), HandClass::HighCard);
        assert_eq!(best_partial_hand_class("Th4s6d3d8cJh"), HandClass::HighCard);
        assert_ne!(best_partial_hand_class("Ah4s4d"), HandClass::HighCard);
    }

    #[test]
    fn pair_class() {
        assert_eq!(best_partial_hand_class("AhAs"), HandClass::Pair);
        assert_eq!(best_partial_hand_class("AhAsJs5h"), HandClass::Pair);
        assert_eq!(best_partial_hand_class("AhAsThJd"), HandClass::Pair);
        assert_eq!(best_partial_hand_class("4h4sAh6s"), HandClass::Pair);
        assert_ne!(best_partial_hand_class("AhAs4h4d"), HandClass::Pair);
    }

    #[test]
    fn two_pair_class() {
        assert_eq!(best_partial_hand_class("AhAs5h5d6s"), HandClass::TwoPair);
        assert_eq!(best_partial_hand_class("AhAs5h6d6s"), HandClass::TwoPair);
        assert_eq!(best_partial_hand_class("AhAs5d5s"), HandClass::TwoPair);
        assert_eq!(best_partial_hand_class("4h4sAsAd"), HandClass::TwoPair);
        // Trips should not match
        assert_ne!(best_partial_hand_class("4h4sAsAd4d"), HandClass::TwoPair);
    }

    #[test]
    fn trips_class() {
        assert_eq!(best_partial_hand_class("AhAs5dAc"), HandClass::ThreeOfAKind);
        assert_eq!(
            best_partial_hand_class("5hAs5d5s4d"),
            HandClass::ThreeOfAKind
        );
        // Full house should not match
        assert_ne!(
            best_partial_hand_class("AhAs5dAc5h"),
            HandClass::ThreeOfAKind
        );
    }

    #[test]
    fn full_house_class() {
        assert_eq!(best_partial_hand_class("AhAsAdKhKs"), HandClass::FullHouse);
        assert_eq!(best_partial_hand_class("2s4h2d4s2c"), HandClass::FullHouse);
        assert_ne!(
            best_partial_hand_class("As2h2dAdAcAh"),
            HandClass::FullHouse
        );
    }

    #[test]
    fn quads_class() {
        assert_eq!(
            best_partial_hand_class("AhAsAdAc5d"),
            HandClass::FourOfAKind
        );
        assert_eq!(
            best_partial_hand_class("5hAs5d5sAhAc5c"),
            HandClass::FourOfAKind
        );
    }

    #[test]
    fn royal_class() {
        let h1 = Hand::from_str("2h3hAhKhQhJhTh").unwrap();
        let hf = h1.finalize_hand();
        dbg!(&hf);
        assert_eq!(
            best_partial_hand_class("2h3hAhKhQhJhTh"),
            HandClass::RoyalFlush
        );
        assert_eq!(best_partial_hand_class("AhKhQhJhTh"), HandClass::RoyalFlush);
        assert_eq!(
            best_partial_hand_class("ThKhQhAhJhAdAs"),
            HandClass::RoyalFlush
        );
    }

    #[test]
    fn straight_class() {
        assert_ne!(best_partial_hand_class("AhKhQhJhTh"), HandClass::Straight);
        assert_ne!(best_partial_hand_class("9hKhQhJhTh"), HandClass::Straight);
        assert_eq!(best_partial_hand_class("9h8h7d6s5c"), HandClass::Straight);
        assert_eq!(best_partial_hand_class("Ah2c3s4d5h"), HandClass::Straight);
        assert_eq!(best_partial_hand_class("6h5d5c4h3d2c"), HandClass::Straight);
        assert_eq!(
            best_partial_hand_class("Ah2c2d3h2s4d5c"),
            HandClass::Straight
        );
    }

    #[test]
    fn straight_flush_class() {
        assert_ne!(
            best_partial_hand_class("AhKhQhJhTh"),
            HandClass::StraightFlush
        );
        assert_eq!(
            best_partial_hand_class("9h8h7h6h5h"),
            HandClass::StraightFlush
        );
        assert_eq!(
            best_partial_hand_class("Ah2h3h4h5h"),
            HandClass::StraightFlush
        );
    }

    #[test]
    fn flush_class() {
        assert_ne!(best_partial_hand_class("AhKhQhJhTh"), HandClass::Flush);
        assert_ne!(best_partial_hand_class("9hKhQhJhTh"), HandClass::Flush);
        assert_eq!(best_partial_hand_class("9h2h5h6hQh"), HandClass::Flush);
        assert_eq!(best_partial_hand_class("Ah2h3h4h8h"), HandClass::Flush);
    }
}

#[cfg(test)]
mod test_runner {
    use super::*;

    fn best_possible_hand_class(s: &'static str) -> HandClass {
        let h = Hand::from_str(s).unwrap();
        //dbg!(&h);
        h.get_best_possible_hand_result()
    }

    #[test]
    fn straight_runner() {
        assert_eq!(
            best_possible_hand_class("2h3h4d5s9cTh"),
            HandClass::Straight
        );
        assert_eq!(best_possible_hand_class("2h3h4d9cTs"), HandClass::Straight);
    }

    #[test]
    fn pair_runner() {
        assert_eq!(best_possible_hand_class("2h4d6s8cTdJs"), HandClass::Pair);
    }

    #[test]
    fn flush_runner() {
        assert_eq!(best_possible_hand_class("2h4h6h8hJsQd"), HandClass::Flush);
        assert_eq!(best_possible_hand_class("2h4h8hJcQd"), HandClass::Flush);
        // Straight Flush potential
        assert_ne!(best_possible_hand_class("2h3h4h5hJdQs"), HandClass::Flush);
    }

    #[test]
    fn quads_runner() {
        assert_eq!(
            best_possible_hand_class("2h2s2d5s6c9h"),
            HandClass::FourOfAKind
        );
    }

    #[test]
    fn full_house_runner() {
        // Two pair + card left
        assert_eq!(
            best_possible_hand_class("AhAsKhKs2d5c"),
            HandClass::FullHouse
        );
    }

    /*#[test]
    fn royal_flush_runner() {
        // Less cards there are make Royal Flush more likely
        assert_eq!(best_possible_hand_class("Ah2d3s"), HandClass::RoyalFlush);
    }*/

    #[test]
    fn straight_flush_runner() {
        assert_eq!(
            best_possible_hand_class("Th9h8h7h2d3c"),
            HandClass::StraightFlush
        );
    }
}

#[cfg(test)]
mod test_wins {
    use super::*;

    #[test]
    fn hand_from_pockets_str() {
        let str = "AhAs";
        let _hand = Hand::from_str(&str).unwrap();
    }

    #[test]
    #[should_panic]
    fn hand_from_one_card() {
        let str = "Ah";
        let _hand = Hand::from_str(&str).unwrap();
    }

    #[test]
    #[should_panic]
    fn hand_from_three_half_card() {
        let str = "AhAsJ";
        let _hand = Hand::from_str(&str).unwrap();
    }

    #[test]
    #[should_panic]
    fn hand_duplicate_cards() {
        let str = "AhAh";
        let _hand = Hand::from_str(&str).unwrap();
    }

    fn best_hand(s: &'static str) -> FinalHandResult {
        Hand::from_str(s).unwrap().finalize_hand()
    }

    fn win_lose(s1: &'static str, s2: &'static str, hc: HandClass) {
        let h1 = Hand::from_str(s1).unwrap().finalize_hand();
        let h2 = Hand::from_str(s2).unwrap().finalize_hand();
        assert_eq!(h1.class, hc);
        assert_eq!(h2.class, hc);
        println!("win? {:?} vs {:?}", h1, h2);
        assert!(h1 > h2);
        println!("lose? {:?} vs {:?}", h2, h1);
        assert!(h2 < h1);
    }

    fn tie(s1: &'static str, s2: &'static str, hc: HandClass) {
        let h1 = Hand::from_str(s1).unwrap().finalize_hand();
        let h2 = Hand::from_str(s2).unwrap().finalize_hand();
        assert_eq!(h1.class, hc);
        assert_eq!(h2.class, hc);
        println!("tie? {:?} vs {:?}", h1, h2);
        assert_eq!(h1, h2);
    }

    #[test]
    fn straight_flush_tie() {
        for (s1, s2) in [
            ("KcQcJcTc9c", "KdQdJdTd9d"),
            ("KcQcJcTc9c", "KdQdJdTd9d"),
            ("5c4c3c2cAc", "5d4d3d2dAd"),
        ] {
            tie(s1, s2, HandClass::StraightFlush);
        }
    }

    #[test]
    fn straight_flush() {
        for (s1, s2) in [
            ("KcQcJcTc9c", "QdJdTd9d8d"),
            ("6c5c4c3c2c", "5d4d3d2dAd"),
            ("KcQcJcTc9c", "5d4d3d2dAd"),
        ] {
            win_lose(s1, s2, HandClass::StraightFlush);
        }
    }

    #[test]
    fn quads_tie() {
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
        for (s1, s2) in [("AsAdKsKdTd", "AcAdKcKdTs")] {
            tie(s1, s2, HandClass::TwoPair);
        }
    }

    #[test]
    fn two_pair() {
        for (s1, s2) in [("AsAdKsKdJd", "AcAdKcKdTs"), ("AsAdKsKdJd", "AcAdQcQdKs")] {
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

    #[test]
    fn best_hand_is_board() {
        let h1 = Hand::from_str("2c3dAdKdQdJdTd").unwrap();
        let h2 = Hand::from_str("3h4h9hThJhQhKh").unwrap();
        dbg!(&h1.get_best_possible_hand_result());
        assert!(h1.playing_board());
        assert!(h2.playing_board());
    }
}

#[cfg(test)]
mod test_straight {
    use super::*;
    use crate::deck::Deck;

    fn get_card_array(s: &'static str) -> Vec<Card> {
        let mut v: Vec<Card> = Vec::new();
        for c in &s.chars().chunks(2) {
            let ss: Vec<char> = c.take(2).collect();
            let ss: [char; 2] = <[char; 2]>::try_from(ss).unwrap();
            v.push(Card::from(ss));
        }
        v.sort();
        v.reverse();
        v
    }

    /// With any random set of cards, if 5 or more cards are yet to come, then
    /// a straight is possible
    #[test]
    fn straight_always_possible_ctc_5() {
        // For many values for cards to come that are all at least 5
        for ctc in [5, 6, 7, 10, 30] {
            // Do many reps (since this is random)
            for _ in 0..100 {
                // For many hand sizes
                for hand_size in [0, 1, 2, 3, 4, 5, 6, 7, 10] {
                    let mut deck = Deck::default();
                    let cards: Vec<_> = (0..hand_size).map(|_| deck.draw()).collect();
                    assert!(is_straight_possible(cards.into_iter(), ctc));
                }
            }
        }
    }

    /// Not enough cards will come in order to make a straight
    #[test]
    fn straight_ctc_1_not_enough() {
        for cards in ["AcKcQc", "9c8c7c", "2c3c4c", "5c", "6c7c"] {
            assert!(!is_straight_possible(get_card_array(&cards).into_iter(), 1));
        }
    }

    /// Not enough cards will come in order to make a straight
    #[test]
    fn straight_ctc_2_not_enough() {
        for cards in ["AcKc", "9c8c", "2c3c", "5c", "6c7c"] {
            assert!(!is_straight_possible(get_card_array(&cards).into_iter(), 2));
        }
    }

    /// Not enough cards will come in order to make a straight
    #[test]
    fn straight_ctc_3_not_enough() {
        for cards in ["Ac", "9c", "2c", "5c", "6c"] {
            assert!(!is_straight_possible(get_card_array(&cards).into_iter(), 3));
        }
    }

    /// Cards to come 1 and there is no gap for a single card to fill that would make a straight
    #[test]
    fn straight_ctc_1_gap() {
        for cards in [
            "AcKcQc",
            "KcQcJc",
            "9c7c6c",
            "AcKcQc3c",
            "4c5c9cTc",
            "2c3c4c9cTcJc",
        ] {
            assert!(!is_straight_possible(get_card_array(&cards).into_iter(), 1));
        }
    }

    /// Cards to come 2 and there is no gap for two cards or two one-card gaps
    #[test]
    fn straight_ctc_2_gap() {
        for cards in [
            "AcKc",
            "KcQc",
            "9c7c",
            "8c7cAc",
            "2c3c9c",
            "2c3c9cTc",
            "2c3c7c8cQcKc",
        ] {
            assert!(!is_straight_possible(get_card_array(&cards).into_iter(), 2));
        }
    }
    /*

    /// Cards to come 3 and there is no combo of gaps for three cards to fill
    /// that would be a straight
    #[test]
    fn straight_ctc_3_gap() {
        unimplemented!();
    }

    /// Blah blah good test for 1 CTC and yes straight is possible
    /// do all: _XXXX X_XXX XX_XX XXX_X XXXX_
    /// make sure to explicitly get A-high and 5-high straights
    #[test]
    fn straight_yes_ctc_1() {
        unimplemented!();
    }

    /// See above, but 2 CTC
    /// Again get all possible positions of those 2 cards
    /// Again explicitly test A-high and 5-high
    #[test]
    fn straight_yes_ctc_2() {
        unimplemented!();
    }

    /// See above, but 3 CTC
    #[test]
    fn straight_yes_ctc_3() {
        unimplemented!();
    }

    /// Should be trivial: give at least 1 card and yes straight possible
    #[test]
    fn straight_yes_ctc_4() {
        unimplemented!();
    }
    */
}
