mod deck;
use deck::{Card, Deck};

fn main() {
    let mut d = Deck::new();
    let c1 = d.draw().unwrap();
    let c2 = d.draw().unwrap();
    println!("{}{}", c1, c2);
}
