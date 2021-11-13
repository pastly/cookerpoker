use poker_core::deck::{Card, Deck};
use poker_core::hand::Hand;

fn main() {
    let mut d = Deck::new();
    let c1 = d.draw().unwrap();
    let c2 = d.draw().unwrap();
    println!("{}{}", c1, c2);

    let cards: Vec<Card> = vec![
        d.draw().unwrap(),
        d.draw().unwrap(),
        d.draw().unwrap(),
        d.draw().unwrap(),
        d.draw().unwrap(),
    ];
    let hand = Hand::new(&cards).unwrap();
    println!("{}", hand);
}
