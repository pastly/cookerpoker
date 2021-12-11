use poker_core::deck::{Card, Rank, Suit};
use wasm_bindgen::JsCast;
use web_sys::Element;

const CARD_BACK: char = '🂠';

/// Implement this trait for types that can be turned into Elements and displayed on a webpage.
pub(crate) trait Elementable {
    /// Consume ourself and turn ourselves into a brand new element.
    ///
    /// The main logic of this function can probably be implemented in fill_element(); give it a
    /// reference to ourself and a new element made here.
    fn into_element(self) -> Element;

    /// Render ourself into the given existing element.
    fn fill_element(&self, elm: &Element);
}

/// Create an Element with the given tag. E.g. with tag "a" create an <a> element.
fn base_element(tag: &str) -> Element {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    doc.create_element(tag)
        .unwrap_or_else(|_| panic!("Unable to create {}", tag))
        .dyn_into::<web_sys::Element>()
        .expect("Unable to dyn_into Element")
}

impl Elementable for Option<Card> {
    fn into_element(self) -> Element {
        let elm = base_element("span");
        self.fill_element(&elm);
        elm
    }

    fn fill_element(&self, elm: &Element) {
        elm.set_class_name("card");
        elm.set_text_content(Some(&format!(
            "{}",
            match self {
                None => CARD_BACK,
                Some(c) => card_char(*c),
            }
        )));
    }
}

pub(crate) struct Community(pub(crate) Vec<Card>);

impl Elementable for Community {
    fn into_element(self) -> Element {
        let elm = base_element("div");
        self.fill_element(&elm);
        elm
    }

    fn fill_element(&self, elm: &Element) {
        while let Some(child) = elm.last_child() {
            elm.remove_child(&child).unwrap();
        }
        for c in &self.0 {
            elm.append_child(&Some(*c).into_element()).unwrap();
        }
    }
}

pub(crate) struct Pot(pub(crate) Option<Vec<i32>>);

impl Elementable for Pot {
    fn into_element(self) -> Element {
        let elm = base_element("div");
        self.fill_element(&elm);
        elm
    }

    fn fill_element(&self, elm: &Element) {
        if self.0.is_none() || self.0.as_ref().unwrap().is_empty() {
            elm.set_text_content(None)
        } else {
            let v = self.0.as_ref().unwrap();
            let mut s = String::from("Pot: ");
            for i in 0..v.len() {
                s += &v[i].to_string();
                if i != v.len() - 1 {
                    s += " | Side pot: ";
                }
            }
            elm.set_text_content(Some(&s))
        }
    }
}

pub(crate) struct Pocket {
    pub(crate) cards: [Option<Card>; 2],
    pub(crate) name: Option<String>,
    pub(crate) monies: Option<i32>,
}

impl Elementable for Pocket {
    fn into_element(self) -> Element {
        let elm = base_element("div");
        self.fill_element(&elm);
        elm
    }

    fn fill_element(&self, elm: &Element) {
        while let Some(child) = elm.last_child() {
            elm.remove_child(&child).unwrap();
        }
        elm.append_child(&self.cards[0].into_element()).unwrap();
        elm.append_child(&self.cards[1].into_element()).unwrap();
        let name_elm = base_element("p");
        let monies_elm = base_element("p");
        if let Some(monies) = self.monies {
            monies_elm.set_text_content(Some(&monies.to_string()));
        }
        if let Some(name) = &self.name {
            name_elm.set_text_content(Some(name));
        }
        elm.append_child(&monies_elm).unwrap();
        elm.append_child(&name_elm).unwrap();
    }
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
