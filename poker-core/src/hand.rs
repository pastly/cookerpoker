use crate::deck::Card;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct Hand {
    cards: [Card; 5],
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
mod tests {
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
