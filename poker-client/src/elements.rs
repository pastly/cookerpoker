use crate::utils::card_char;
use poker_core::bet::BetStatus;
use poker_core::cards::{card::Suit, Card};
use poker_core::PlayerId;
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
        if let Some(c) = self {
            let suit_class = match c.suit {
                Suit::Club => "card-club",
                Suit::Diamond => "card-diamond",
                Suit::Heart => "card-heart",
                Suit::Spade => "card-spade",
            };
            elm.class_list()
                .add_1(suit_class)
                .expect("unable to add suit-specific class to card");
        }
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

pub(crate) struct Pot(pub(crate) Vec<i32>);

impl Elementable for Pot {
    fn into_element(self) -> Element {
        let elm = base_element("div");
        self.fill_element(&elm);
        elm
    }

    fn fill_element(&self, elm: &Element) {
        let v = &self.0;
        if v.is_empty() {
            elm.set_text_content(None);
            return;
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pocket {
    // lots of option on cards because want to be able to represent lots of things:
    // - player sitting but no cards (yet): None
    // - player sitting and has unknown cards: Some([None, None])
    // - player sitting and has revealed a card: Some([Some(), None])
    // - player sitting and either has revealed both cards or its us: Some([Some(), Some()])
    pub(crate) cards: Option<[Option<Card>; 2]>,
    pub(crate) name: String,
    pub(crate) stack: i32,
    pub(crate) seat_idx: usize,
    pub(crate) player_id: PlayerId,
    pub(crate) bet_status: BetStatus,
    pub(crate) is_btn: bool,
    pub(crate) is_sb: bool,
    pub(crate) is_bb: bool,
    pub(crate) needs_better_name: bool,
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
        if self.cards.is_some() {
            elm.append_child(&self.cards.unwrap()[0].into_element())
                .unwrap();
            elm.append_child(&self.cards.unwrap()[1].into_element())
                .unwrap();
        }
        let name_elm = base_element("p");
        let stack_elm = base_element("p");
        stack_elm.set_text_content(Some(&self.stack.to_string()));
        name_elm.set_text_content(Some(&self.name));
        elm.append_child(&stack_elm).unwrap();
        elm.append_child(&name_elm).unwrap();
        let token_elm = base_element("p");
        let mut tokens = vec![];
        if self.is_btn {
            tokens.push("BTN");
        }
        if self.is_sb {
            tokens.push("SB");
        }
        if self.is_bb {
            tokens.push("BB");
        }
        token_elm.set_text_content(Some(&tokens.join("/")));
        elm.append_child(&token_elm).unwrap();
        match self.bet_status {
            BetStatus::Folded | BetStatus::Waiting => {}
            BetStatus::In(x) | BetStatus::AllIn(x) => {
                if x > 0 {
                    let wager_elm = base_element("p");
                    wager_elm.set_text_content(Some(&format!("Wager: {x}")));
                    elm.append_child(&wager_elm).unwrap();
                }
            }
        }
    }
}
