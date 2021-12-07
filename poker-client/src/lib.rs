mod utils;

use poker_core::deck::{Card, Deck, Rank, Suit};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, Node};

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
pub fn show_n_cards(elm: Node, n: u8) {
    while let Some(child) = elm.last_child() {
        elm.remove_child(&child).unwrap();
    }
    let mut d = Deck::default();
    for _ in 0..n {
        let c = d.draw().unwrap();
        let card_elm = new_card_elm();
        card_elm.set_text_content(Some(&format!("{}", card_char(c))));
        elm.append_child(&card_elm).unwrap();
    }
}

fn new_card_elm() -> Element {
    let doc = web_sys::window().unwrap().document().unwrap();
    let elm = doc
        .create_element("span")
        .unwrap()
        .dyn_into::<web_sys::Element>()
        .map_err(|_| ())
        .unwrap();
    elm.set_class_name("card");
    elm
}

fn card_char(card: Card) -> char {
    // https://en.wikipedia.org/wiki/Playing_cards_in_Unicode#Block
    let base: u32 = match card.suit() {
        Suit::Spade => 0x1F0A0,
        Suit::Heart => 0x1F0B0,
        Suit::Diamond => 0x1F0C0,
        Suit::Club => 0x1F0D0,
    };
    let val = base
        + match card.rank() {
            Rank::RA => 1,
            Rank::R2 => 2,
            Rank::R3 => 3,
            Rank::R4 => 4,
            Rank::R5 => 5,
            Rank::R6 => 6,
            Rank::R7 => 7,
            Rank::R8 => 8,
            Rank::R9 => 9,
            Rank::RT => 10,
            Rank::RJ => 11,
            // Unicode includes Knight here. Skip 12.
            Rank::RQ => 13,
            Rank::RK => 14,
        };
    // Safety: Value will always be a valid char thanks to match statements and enums on card
    // suits and ranks.
    unsafe { std::char::from_u32_unchecked(val) }
}

fn _char_card(c: char) -> Option<Card> {
    let c = c as u32;
    let (base, suit): (u32, _) = {
        if c > 0x1F0D0 {
            (0x1F0D0, Suit::Club)
        } else if c > 0x1F0C0 {
            (0x1F0C0, Suit::Diamond)
        } else if c > 0x1F0B0 {
            (0x1F0B0, Suit::Heart)
        } else if c > 0x1F0A0 {
            (0x1F0A0, Suit::Spade)
        } else {
            return None;
        }
    };
    let rank = {
        let diff = c - base;
        match diff {
            1 => Rank::RA,
            2 => Rank::R2,
            3 => Rank::R3,
            4 => Rank::R4,
            5 => Rank::R5,
            6 => Rank::R6,
            7 => Rank::R7,
            8 => Rank::R8,
            9 => Rank::R9,
            10 => Rank::RT,
            11 => Rank::RJ,
            // Unicode includes Knight here. Skip 12.
            13 => Rank::RQ,
            14 => Rank::RK,
            _ => return None,
        }
    };
    Some(Card::new(rank, suit))
}
