use poker_core::deck::Deck;
use poker_core::hand::best_of_cards;

fn main() {
    let n = 7;
    let reps = 1;
    for rep in 0..reps {
        let mut d = Deck::default();
        let cards: Vec<_> = (0..n).map(|_| d.draw().unwrap()).collect();
        for item in best_of_cards(&cards) {
            println!("{} {}", rep, item);
        }
    }
}
