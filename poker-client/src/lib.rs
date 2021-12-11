mod elements;
mod utils;

use elements::{Community, Elementable, Pocket, Pot};
use poker_core::deck::Deck;
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

#[wasm_bindgen]
pub fn greet() {
    utils::set_panic_hook();
    alert("Hello, poker-client!");
}

#[wasm_bindgen]
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

#[wasm_bindgen]
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

#[wasm_bindgen]
pub fn show_pot() {
    let pot = Pot(Some(vec![100, 450, 420]));
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id("pot").unwrap();
    pot.fill_element(&elm);
}
