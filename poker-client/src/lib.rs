mod actionlog;
mod elements;
pub mod http;
mod utils;

use elements::{Community, Elementable, Pocket, Pot};
use poker_core::deck::Deck;
use poker_core::game::BetAction;
use poker_messages::*;
use std::time::Duration;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

//#[wasm_bindgen]
pub fn greet() {
    utils::set_panic_hook();
    alert("Hello, poker-client!");
}

//#[wasm_bindgen]
pub fn show_community(n: u8) {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id("community").unwrap();
    let mut d = Deck::default();
    let cards = (0..n).map(|_| d.draw().unwrap()).collect();
    let comm = Community(cards);
    comm.fill_element(&elm);
}

//#[wasm_bindgen]
pub fn show_pocket(seat: u8) {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id(&format!("pocket-{}", seat)).unwrap();
    let mut d = Deck::default();
    let pocket = Pocket {
        cards: [Some(d.draw().unwrap()), None],
        name: Some(String::from("Matt")),
        monies: Some(42069),
    };
    pocket.fill_element(&elm);
}

//#[wasm_bindgen]
pub fn show_pot() {
    let pot = Pot(Some(vec![100, 450, 420]));
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id("pot").unwrap();
    pot.fill_element(&elm);
}

fn next_seq_num() -> SeqNum {
    static mut LAST_SEQ_NUM: SeqNum = 0;
    unsafe {
        LAST_SEQ_NUM += 1;
        LAST_SEQ_NUM
    }
}

fn a(ae: ActionEnum) -> Action {
    Action {
        seq: next_seq_num(),
        action: ae,
    }
}

//#[wasm_bindgen]
pub fn render() {
    let mut d = Deck::default();
    let p1 = PlayerInfo::new(1001, "Alice".to_string(), 5000, 1);
    let p2 = PlayerInfo::new(1002, "Bob".to_string(), 5000, 2);
    let p3 = PlayerInfo::new(1003, "Charlie".to_string(), 5000, 3);
    let p4 = PlayerInfo::new(1004, "David".to_string(), 5000, 4);
    let mut actions = vec![a(ActionEnum::Epoch(Epoch::new(
        vec![p1, p2, p3, p4],
        (5, 10),
        (1, 2, 3),
        Duration::new(15, 0),
    )))];
    actions.push(a(ActionEnum::CardsDealt(CardsDealt::new(
        vec![1, 2, 3, 4],
        [d.draw().unwrap(), d.draw().unwrap()],
    ))));
    actions.push(a(ActionEnum::Bet(Bet::new(2, BetAction::Check))));
    actions.push(a(ActionEnum::Flop(Flop([
        d.draw().unwrap(),
        d.draw().unwrap(),
        d.draw().unwrap(),
    ]))));
    actions.push(a(ActionEnum::Turn(Turn(d.draw().unwrap()))));
    actions.push(a(ActionEnum::River(River(d.draw().unwrap()))));
    actions.push(a(ActionEnum::Reveal(Reveal::new(
        3,
        [d.draw().unwrap(), d.draw().unwrap()],
    ))));
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id("gamelog").unwrap();
    actionlog::render_html_list(&ActionList(actions), &elm, 1001).unwrap();
}
