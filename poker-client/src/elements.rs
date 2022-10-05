use crate::utils::card_char;
use poker_core::deck::Card;
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
    pub(crate) stack: Option<i32>,
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
        let stack_elm = base_element("p");
        if let Some(stack) = self.stack {
            stack_elm.set_text_content(Some(&stack.to_string()));
        }
        if let Some(name) = &self.name {
            name_elm.set_text_content(Some(name));
        }
        elm.append_child(&stack_elm).unwrap();
        elm.append_child(&name_elm).unwrap();
    }
}
