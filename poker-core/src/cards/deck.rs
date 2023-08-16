use base64ct::{self, Base64, Encoding};
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::str::FromStr;

use super::card::{all_cards, Card};
use super::fill_random;

const SEED_LEN: usize = 32;
const ENCODED_SEED_LEN: usize = 4 * ((SEED_LEN + 3 - 1) / 3); // 4 * ceil(SEED_LEN / 3)
pub type GameRng = ChaChaRng;

/// A `Deck` will always be shuffled according to the seed provided at initialization
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq)]
pub struct Deck {
    #[serde(skip)]
    #[serde(default = "all_cards")]
    cards: [Card; 52],
    index: usize,
    pub seed: DeckSeed,
    /// This will only ever be false when deserialized
    #[serde(skip)]
    sorted: bool,
}

impl std::cmp::PartialEq for Deck {
    fn eq(&self, other: &Self) -> bool {
        self.cards.eq(&other.cards)
    }
}

impl std::default::Default for Deck {
    fn default() -> Self {
        let seed = DeckSeed::default();
        Self::new(seed)
    }
}

impl Deck {
    pub fn new(seed: DeckSeed) -> Self {
        let mut cards = all_cards();
        let mut rng = ChaChaRng::from_seed(*seed);
        cards.shuffle(&mut rng);
        Deck {
            cards,
            index: 0,
            seed,
            sorted: true,
        }
    }

    pub fn can_draw(&self) -> bool {
        self.index < 52
    }

    /// Helper function to deal out many cards at once
    pub fn deal_pockets(&mut self, num_players: u8) -> Vec<[Card; 2]> {
        let mut v = Vec::new();
        for _ in 0..num_players {
            let c1 = self.draw();
            let c2 = self.draw();
            v.push([c1, c2]);
        }
        v
    }
    /// Returns a card and increments the deck index
    /// # Panics
    /// Panics if index is out of bounds. i.e. this function is called 53 times on the same deck
    /// At that point the game is clearly in an unrecoverable state.
    pub fn draw(&mut self) -> Card {
        // This will only run on the first draw
        if !self.sorted {
            let mut rng = ChaChaRng::from_seed(*self.seed);
            self.cards.shuffle(&mut rng);
            for _ in 0..self.index {
                self.burn();
            }
        }
        if self.index >= 52 {
            panic!("No cards left to draw!")
        }
        let c = self.cards[self.index];
        self.index += 1;
        c
    }

    pub fn burn(&mut self) {
        self.draw();
    }
    #[cfg(test)]
    #[allow(dead_code)]
    /// While running tests it's useful to have the raw deck order
    fn get_cards(self) -> [Card; 52] {
        self.cards
    }

    #[cfg(test)]
    #[allow(dead_code)]
    /// Used to validate serialization/deserialization
    fn previous_card(&self) -> Card {
        self.cards[self.index - 1]
    }
}

#[derive(Clone, Copy, Debug, derive_more::Display, PartialEq, Eq, Serialize, Deserialize)]
#[display(fmt = "{:?}", "self.0")]
pub struct DeckSeed([u8; SEED_LEN]);

impl FromStr for DeckSeed {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut value = [0u8; SEED_LEN];
        Base64::decode(s, &mut value).map_err(|_| "Failed to decode")?;
        Ok(DeckSeed(value))
    }
}

impl Deref for DeckSeed {
    type Target = [u8; SEED_LEN];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::default::Default for DeckSeed {
    fn default() -> DeckSeed {
        DeckSeed(fill_random::<SEED_LEN>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_same_seeds() {
        let d = Deck::default();
        let s = d.seed;
        let d2 = Deck::new(s);
        let d3 = Deck::new(s);
        assert_eq!(d, d2);
        assert_eq!(d, d3);
    }

    #[test]
    fn test_different_seeds() {
        let d = Deck::default();
        let d2 = Deck::default();
        let d3 = Deck::default();
        assert_ne!(d, d2);
        assert_ne!(d, d3);
    }

    /*#[test]
    fn all_cards_unique() {
        for _ in 0..1000 {
            let d = Deck::default();
            let c = d.get_cards();
            let mut c: Vec<Card> = c.into_iter().collect();
            // Obviously this looks dumb
            // Since PartialEq is implemented over only rank, we expect a result for each rank, not each rank + suit
            // As such, sort + dedup should always = number of ranks
            // A more proper test was harder than I had energy for
            // And it doesn't even work because I don't want Card = Eq
            //c.sort_unstable();
            c.dedup();
            assert_eq!(c.len(), 13);
        }
    }*/

    #[test]
    fn can_draw_52() {
        let mut d = Deck::default();
        for _ in 0..52 {
            d.draw();
        }
        assert_eq!(d.can_draw(), false)
    }

    #[test]
    #[should_panic]
    fn cannot_draw_53() {
        let mut d = Deck::default();
        for _ in 0..53 {
            d.draw();
        }
        // This line is never reached
    }

    #[test]
    fn serde() {
        let mut d = Deck::default();
        let c = d.draw();
        d.burn();
        d.burn();
        let v = serde_json::to_string(&d).unwrap();
        let mut d2: Deck = serde_json::from_str(&v).unwrap();
        let c2 = d2.previous_card();
        assert_eq!(c, c2);
    }
}
