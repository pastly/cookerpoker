mod actionlog;
mod elements;
pub mod http;
mod utils;

use elements::{Community, Elementable, Pocket, Pot};
use poker_core::deck::Deck;
use poker_core::game::BetAction;
use poker_messages::game::*;
use poker_messages::table_mgmt::*;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlInputElement};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const K_DEV_TABLE_N: &str = "dev-table-n";
const K_DEV_PLAYER_N: &str = "dev-player-n";
const K_DEV_PLAYER_BALANCE: &str = "dev-player-balance";

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

#[wasm_bindgen]
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

fn dev_controls_vars(main: &Element) {
    let table_n_label = base_element("label")
        .dyn_into::<web_sys::HtmlLabelElement>()
        .expect("Unable to dyn_into HtmlLabelElement");
    table_n_label.set_inner_text("Table number");
    table_n_label.set_html_for(K_DEV_TABLE_N);
    let table_n_input = base_element("input")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("Unable to dyn_into HtmlInputElement");
    table_n_input.set_type("number");
    table_n_input.set_id(K_DEV_TABLE_N);
    table_n_input.set_value_as_number(1f64);
    main.append_child(&table_n_label).unwrap();
    main.append_child(&table_n_input).unwrap();

    let player_n_label = base_element("label")
        .dyn_into::<web_sys::HtmlLabelElement>()
        .expect("Unable to dyn_into HtmlLabelElement");
    player_n_label.set_inner_text("Player number");
    player_n_label.set_html_for(K_DEV_PLAYER_N);
    let player_n_input = base_element("input")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("Unable to dyn_into HtmlInputElement");
    player_n_input.set_type("number");
    player_n_input.set_id(K_DEV_PLAYER_N);
    player_n_input.set_value_as_number(1f64);
    main.append_child(&player_n_label).unwrap();
    main.append_child(&player_n_input).unwrap();

    let player_balance_label = base_element("label")
        .dyn_into::<web_sys::HtmlLabelElement>()
        .expect("Unable to dyn_into HtmlLabelElement");
    player_balance_label.set_inner_text("Player balance");
    let player_balance_input = base_element("input")
        .dyn_into::<web_sys::HtmlInputElement>()
        .expect("Unable to dyn_into HtmlInputElement");
    player_balance_input.set_type("number");
    player_balance_input.set_id(K_DEV_PLAYER_BALANCE);
    player_balance_input.set_value_as_number(1000f64);
    main.append_child(&player_balance_label).unwrap();
    main.append_child(&player_balance_input).unwrap();
}

fn dev_controls_buttons(main: &Element) {
    let sit_btn = base_element("button");
    sit_btn.set_text_content(Some("Sit player N at table N"));
    main.append_child(&sit_btn).unwrap();

    let send_sit_closure = Closure::wrap(Box::new(move || {
        let doc = web_sys::window()
            .expect("No window?")
            .document()
            .expect("No document?");
        let table_n = doc
            .get_element_by_id(K_DEV_TABLE_N)
            .expect("Unable to find table input")
            .dyn_into::<HtmlInputElement>()
            .expect("Unable to dyn_into HtmlInputElement")
            .value_as_number() as i32;
        let player_n = doc
            .get_element_by_id(K_DEV_PLAYER_N)
            .expect("Unable to find table input")
            .dyn_into::<HtmlInputElement>()
            .expect("Unable to dyn_into HtmlInputElement")
            .value_as_number() as i32;
        let player_balance = doc
            .get_element_by_id(K_DEV_PLAYER_BALANCE)
            .expect("Unable to find table input")
            .dyn_into::<HtmlInputElement>()
            .expect("Unable to dyn_into HtmlInputElement")
            .value_as_number() as i32;
        let msg = SitIntent::new(player_n, table_n, player_balance);
        alert(&serde_json::to_string(&msg).unwrap());
    }) as Box<dyn FnMut()>);

    sit_btn
        .dyn_ref::<HtmlElement>()
        .expect("button should be HtmlElement")
        .set_onclick(Some(send_sit_closure.as_ref().unchecked_ref()));

    send_sit_closure.forget();
}

#[wasm_bindgen]
pub fn create_dev_controls() {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let main_div = doc.get_element_by_id("devcontrols").unwrap();
    while let Some(child) = main_div.last_child() {
        main_div.remove_child(&child).unwrap();
    }
    let vars_div = base_element("div");
    let btns_div = base_element("div");
    dev_controls_vars(&vars_div);
    dev_controls_buttons(&btns_div);
    main_div.append_child(&vars_div).unwrap();
    main_div.append_child(&btns_div).unwrap();
}
