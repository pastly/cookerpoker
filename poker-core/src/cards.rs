use rand::Fill;

pub mod card;
pub mod deck;
pub mod hand;

pub use card::Card;
pub use deck::Deck;
pub use hand::{Hand, HandSolver};

fn fill_random<const L: usize>() -> [u8; L] {
    let mut r = rand::thread_rng();
    let mut s: [u8; L] = [0; L];
    s.try_fill(&mut r)
        .expect("Failed to generate random numbers");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_random() {
        let a: [u8; 32] = fill_random();
        let m: u8 = a.into_iter().max().unwrap();
        assert_ne!(m, 0);
    }
}
