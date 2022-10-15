#![allow(clippy::unused_unit)]
//mod actionlog;
mod elements;
mod utils;

use elements::{Community, Elementable, Pocket, Pot};
use poker_core::bet::BetStatus;
use poker_core::deck::{Card, Deck};
use poker_core::player::Player;
use poker_core::state::FilteredGameState;
use poker_core::Currency;
use poker_messages::{action, Msg};
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlInputElement};

#[macro_use]
extern crate lazy_static;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    static ref LAST_STATE: Mutex<Option<FilteredGameState>> = Mutex::new(None);
}
//const K_DEV_TABLE_N: &str = "dev-table-n";
//const K_DEV_PLAYER_N: &str = "dev-player-n";
//const K_DEV_PLAYER_BALANCE: &str = "dev-player-balance";

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    fn send_action(s: &str);
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
pub fn show_pot() {
    let pot = Pot(Some(vec![100, 450, 420]));
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id("pot").unwrap();
    pot.fill_element(&elm);
}

fn redraw_pocket(elm: &HtmlElement, player: &Player, _is_cash: bool) {
    let p = Pocket {
        cards: [
            if player.pocket.is_some() {
                Some(player.pocket.unwrap()[0])
            } else {
                None
            },
            if player.pocket.is_some() {
                Some(player.pocket.unwrap()[1])
            } else {
                None
            },
        ],
        name: Some(format!("Player {}", player.id)),
        stack: Some(player.stack),
    };
    p.fill_element(elm);
}

fn redraw_table(state: &FilteredGameState) {
    let mut next_player_div = 1;
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    for (idx, player) in state.players.players_iter_with_index() {
        let div_id = format!("pocket-{}", next_player_div);
        next_player_div += 1;
        let elm = doc.get_element_by_id(&div_id).unwrap();
        redraw_pocket(
            elm.dyn_ref::<HtmlElement>()
                .expect("div should be HtmlElement"),
            player,
            state.is_cash(),
        );
        if state.nta_seat.is_some() && state.nta_seat.unwrap() == idx {
            elm.class_list().add_1("next-action").unwrap();
        } else {
            elm.class_list().remove_1("next-action").unwrap();
        }
        if idx == state.players.token_dealer {
            let p = base_element("p");
            p.set_text_content(Some("BTN"));
            elm.dyn_ref::<HtmlElement>()
                .expect("HtmlElement")
                .append_child(&p)
                .unwrap();
        }
    }
    let community_elm = doc.get_element_by_id("community").unwrap();
    let community: Vec<Card> = state
        .community
        .iter()
        .take_while(|c| c.is_some())
        .map(|c| c.unwrap())
        .collect();
    Community(community).fill_element(&community_elm);
    let pot_elm = doc.get_element_by_id("pot").unwrap();
    Pot(state.pot.clone()).fill_element(&pot_elm);
}

fn redraw_logs(logs: &[poker_core::log::LogItem]) {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let logs_div = doc.get_element_by_id("logs").unwrap();
    while let Some(child) = logs_div.last_child() {
        logs_div.remove_child(&child).unwrap();
    }
    for log in logs.iter() {
        let p = base_element("p");
        p.set_text_content(Some(&format!("{}", log)));
        logs_div.append_child(&p).unwrap();
    }
}

fn redraw_state(state: &FilteredGameState) {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let state_div = doc.get_element_by_id("state").unwrap();
    state_div.set_text_content(Some(&serde_json::to_string_pretty(&state).unwrap()));
}

fn get_self(state: &FilteredGameState) -> Option<&Player> {
    state.players.player_by_id(state.self_id)
}

fn is_self_nta(state: &FilteredGameState) -> bool {
    state.nta_seat == state.self_seat
}

fn redraw_action_buttons(state: &FilteredGameState) {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id("action-buttons").unwrap();
    while let Some(child) = elm.last_child() {
        elm.remove_child(&child).unwrap();
    }
    if !is_self_nta(state) {
        return;
    }
    let player_self = get_self(state).expect("No self");
    let bet_status = player_self.bet_status;
    let stack = player_self.stack;
    let call_amount = match bet_status {
        BetStatus::Folded | BetStatus::AllIn(_) => 0,
        BetStatus::Waiting => state.current_bet,
        BetStatus::In(x) => {
            if x < state.current_bet {
                state.current_bet - x
            } else {
                0
            }
        }
    };
    let can_fold = call_amount > 0;
    if can_fold {
        let btn = base_element("button");
        btn.set_text_content(Some("Fold"));
        btn.set_attribute("onclick", "onclick_fold()").unwrap();
        elm.append_child(&btn).unwrap();
    }
    let can_check = call_amount <= 0;
    if can_check {
        let btn = base_element("button");
        btn.set_text_content(Some("Check"));
        btn.set_attribute("onclick", "onclick_check()").unwrap();
        elm.append_child(&btn).unwrap();
    }
    let can_call = call_amount > 0;
    if can_call {
        let btn = base_element("button");
        btn.set_text_content(Some(&format!("Call ({})", call_amount)));
        btn.set_attribute("onclick", "onclick_call()").unwrap();
        elm.append_child(&btn).unwrap();
    }
    // you can always either bet or raise, but not both.
    let is_bet = call_amount <= 0 && state.community[0].is_some();
    let (label, func) = if is_bet {
        ("Bet", "onclick_bet()")
    } else {
        ("Raise", "onclick_raise()")
    };
    let btn = base_element("button");
    btn.set_text_content(Some(label));
    btn.set_attribute("onclick", func).unwrap();
    elm.append_child(&btn).unwrap();
    let min_raise = if stack < state.min_raise {
        stack
    } else {
        state.min_raise
    };
    let max_raise = stack;
    let slider = base_element("input")
        .dyn_into::<HtmlInputElement>()
        .expect("HtmlInputElement");
    slider.set_type("range");
    slider.set_min(&min_raise.to_string());
    slider.set_max(&max_raise.to_string());
    slider.set_value(&min_raise.to_string());
    slider.set_id("raise-slider");
    slider
        .set_attribute("onchange", "onchange_raise(this.value)")
        .unwrap();
    let box_ = base_element("input")
        .dyn_into::<HtmlInputElement>()
        .expect("HtmlInputElement");
    box_.set_type("number");
    box_.set_min(&min_raise.to_string());
    box_.set_max(&max_raise.to_string());
    box_.set_value(&min_raise.to_string());
    box_.set_id("raise-box");
    box_.set_attribute("oninput", "onchange_raise(this.value)")
        .unwrap();
    elm.append_child(&slider).unwrap();
    elm.append_child(&box_).unwrap();
}

/// Redraw the table/hands/etc. based on the given state object. Return the number of seconds we
/// should wait before polling for a new update.
#[wasm_bindgen]
pub fn redraw(state: String) -> i32 {
    let state: FilteredGameState = serde_json::from_str(&state).unwrap();
    let mut last_state = LAST_STATE.lock().expect("could not get last state");
    if last_state.is_some() && *last_state.as_ref().unwrap() == state {
        return if is_self_nta(&state) { 30 } else { 2 };
    }
    *last_state = Some(state.clone());
    redraw_table(&state);
    redraw_logs(&state.logs);
    redraw_state(&state);
    redraw_action_buttons(&state);
    2
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

#[wasm_bindgen]
pub fn onclick_fold() {
    let msg = Msg::Action(action::Msg::Fold);
    send_action(&serde_json::to_string(&msg).unwrap());
}

#[wasm_bindgen]
pub fn onclick_call() {
    let msg = Msg::Action(action::Msg::Call);
    send_action(&serde_json::to_string(&msg).unwrap());
}

#[wasm_bindgen]
pub fn onclick_check() {
    let msg = Msg::Action(action::Msg::Check);
    send_action(&serde_json::to_string(&msg).unwrap());
}

#[wasm_bindgen]
pub fn onclick_bet() {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let box_ = doc
        .get_element_by_id("raise-box")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .expect("HtmlInputElement");
    let v = box_.value_as_number() as Currency;
    let msg = Msg::Action(action::Msg::Bet(v));
    send_action(&serde_json::to_string(&msg).unwrap());
}

#[wasm_bindgen]
pub fn onclick_raise() {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let box_ = doc
        .get_element_by_id("raise-box")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .expect("HtmlInputElement");
    let v = box_.value_as_number() as Currency;
    let msg = Msg::Action(action::Msg::Raise(v));
    send_action(&serde_json::to_string(&msg).unwrap());
}

#[wasm_bindgen]
pub fn onchange_raise(val: f64) {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let raise_box = doc
        .get_element_by_id("raise-box")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .expect("HtmlInputElement");
    raise_box.set_value_as_number(val);
    let raise_slider = doc
        .get_element_by_id("raise-slider")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .expect("HtmlInputElement");
    raise_slider.set_value_as_number(val);
}
