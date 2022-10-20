#![allow(clippy::unused_unit)]
//mod actionlog;
mod elements;
mod player_info;
mod utils;

use elements::{Community, Elementable, Pocket, Pot};
use player_info::PlayerInfo;
use poker_core::bet::BetStatus;
use poker_core::deck::{Card, Deck};
use poker_core::log::LogItem;
use poker_core::pot;
use poker_core::{Currency, PlayerId, SeqNum, MAX_PLAYERS};
use poker_messages::{action, Msg};
use std::collections::HashMap;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};

#[macro_use]
extern crate lazy_static;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

lazy_static! {
    static ref SAVED_LOGS: Mutex<Vec<(usize, LogItem)>> = Mutex::new(Vec::new());
    static ref POCKETS: Mutex<Vec<Pocket>> = Mutex::new(Vec::with_capacity(MAX_PLAYERS));
    static ref COMMUNITY: Mutex<[Option<Card>; 5]> = Mutex::new([None; 5]);
    static ref CURRENT_BET_AND_RAISE: Mutex<(Currency, Currency)> = Mutex::new((0, 0));
    static ref NTA: Mutex<usize> = Mutex::new(MAX_PLAYERS+1);
    static ref POT: Mutex<Vec<Currency>> = Mutex::new(Vec::with_capacity(4));
    static ref PLAYER_INFO: Mutex<HashMap<PlayerId, PlayerInfo>> = Mutex::new(HashMap::new());
}
//const K_DEV_TABLE_N: &str = "dev-table-n";
//const K_DEV_PLAYER_N: &str = "dev-player-n";
//const K_DEV_PLAYER_BALANCE: &str = "dev-player-balance";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    fn alert(s: &str);
    fn send_action(last_seq: SeqNum, s: &str);
    fn send_player_info_request(player_id: PlayerId);
    fn self_player_id() -> PlayerId;
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
    let pot = Pot(vec![100, 450, 420]);
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id("pot").unwrap();
    pot.fill_element(&elm);
}

//fn redraw_pocket(elm: &HtmlElement, player: &Player, _is_cash: bool) {
//    let player_info_cache = PLAYER_INFO
//        .lock()
//        .expect("Unable to lock player info cache");
//    let name = player_info_cache
//        .get(&player.id)
//        .map(|pi| pi.username.clone())
//        .unwrap_or_else(|| format!("Player {}", player.id));
//    let p = Pocket {
//        cards: Some([
//            if player.pocket.is_some() {
//                Some(player.pocket.unwrap()[0])
//            } else {
//                None
//            },
//            if player.pocket.is_some() {
//                Some(player.pocket.unwrap()[1])
//            } else {
//                None
//            },
//        ]),
//        name: Some(name),
//        stack: Some(player.stack),
//    };
//    p.fill_element(elm);
//}

//fn redraw_table(state: &FilteredGameState) {
//    let mut next_player_div = 1;
//    let doc = web_sys::window()
//        .expect("No window?")
//        .document()
//        .expect("No document?");
//    for (idx, player) in state.players.players_iter_with_index() {
//        let div_id = format!("pocket-{}", next_player_div);
//        next_player_div += 1;
//        let elm = doc.get_element_by_id(&div_id).unwrap();
//        redraw_pocket(
//            elm.dyn_ref::<HtmlElement>()
//                .expect("div should be HtmlElement"),
//            player,
//            state.is_cash(),
//        );
//        if state.nta_seat.is_some() && state.nta_seat.unwrap() == idx {
//            elm.class_list().add_1("next-action").unwrap();
//        } else {
//            elm.class_list().remove_1("next-action").unwrap();
//        }
//        if idx == state.players.token_dealer {
//            let p = base_element("p");
//            p.set_text_content(Some("BTN"));
//            elm.dyn_ref::<HtmlElement>()
//                .expect("HtmlElement")
//                .append_child(&p)
//                .unwrap();
//        }
//    }
//    let community_elm = doc.get_element_by_id("community").unwrap();
//    let community: Vec<Card> = state
//        .community
//        .iter()
//        .take_while(|c| c.is_some())
//        .map(|c| c.unwrap())
//        .collect();
//    Community(community).fill_element(&community_elm);
//    let pot_elm = doc.get_element_by_id("pot").unwrap();
//    Pot(state.pot.clone()).fill_element(&pot_elm);
//}

//fn redraw_logs(logs: &[poker_core::log::LogItem]) {
//    let doc = web_sys::window()
//        .expect("No window?")
//        .document()
//        .expect("No document?");
//    let logs_div = doc.get_element_by_id("logs").unwrap();
//    while let Some(child) = logs_div.last_child() {
//        logs_div.remove_child(&child).unwrap();
//    }
//    for log in logs.iter() {
//        let p = base_element("p");
//        p.set_text_content(Some(&format!("{}", log)));
//        logs_div.append_child(&p).unwrap();
//    }
//}

//fn redraw_state(state: &FilteredGameState) {
//    let doc = web_sys::window()
//        .expect("No window?")
//        .document()
//        .expect("No document?");
//    let state_div = doc.get_element_by_id("state").unwrap();
//    state_div.set_text_content(Some(&serde_json::to_string_pretty(&state).unwrap()));
//}

fn get_self_pocket(pockets: &[Pocket]) -> Option<&Pocket> {
    pockets
        .iter()
        .find(|&pocket| pocket.player_id == self_player_id())
}

fn is_self_nta() -> bool {
    let nta = NTA.lock().expect("unable to get saved nta");
    let pid = self_player_id();
    let pockets = POCKETS.lock().expect("unable to get saved pockets");
    for pocket in pockets.iter() {
        if pocket.player_id == pid {
            return pocket.seat_idx == *nta;
        }
    }
    false
}

fn redraw_action_buttons(action_on_self: bool) {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id("action-buttons").unwrap();
    while let Some(child) = elm.last_child() {
        elm.remove_child(&child).unwrap();
    }
    if !action_on_self {
        return;
    }
    let seen_flop = COMMUNITY
        .lock()
        .expect("unable to get saved community cards")[1]
        .is_some();
    let pockets = POCKETS.lock().expect("unable to get saved pockets");
    let (current_bet, current_min_raise) = {
        let res = CURRENT_BET_AND_RAISE
            .lock()
            .expect("unable to get saved current bet");
        (res.0, res.1)
    };
    let pocket_self = get_self_pocket(&pockets).expect("No self");
    let bet_status = pocket_self.bet_status;
    let stack = pocket_self.stack;
    let call_amount = match bet_status {
        BetStatus::Folded | BetStatus::AllIn(_) => 0,
        BetStatus::Waiting => current_bet,
        BetStatus::In(x) => {
            if x < current_bet {
                current_bet - x
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
    let is_bet = call_amount <= 0 && seen_flop;
    let (label, func) = if is_bet {
        ("Bet", "onclick_bet()")
    } else {
        ("Raise", "onclick_raise()")
    };
    let btn = base_element("button");
    btn.set_text_content(Some(label));
    btn.set_attribute("onclick", func).unwrap();
    elm.append_child(&btn).unwrap();
    let min_raise = if stack < current_min_raise {
        stack
    } else {
        current_min_raise
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

//fn send_player_info_requests_for_missing_players(state: &FilteredGameState) {
//    let cache = PLAYER_INFO
//        .lock()
//        .expect("could not lock player info cache");
//    for pid in state.players.players_iter().map(|p| p.id) {
//        if !cache.contains_key(&pid) {
//            send_player_info_request(pid);
//        }
//    }
//}

fn redraw_pockets() {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    for n in 0..MAX_PLAYERS {
        let elm_id = format!("pocket-{}", n);
        if let Some(elm) = doc.get_element_by_id(&elm_id) {
            while let Some(child) = elm.last_child() {
                elm.remove_child(&child).unwrap();
            }
        }
    }
    let pockets = POCKETS.lock().expect("could not get saved pockets");
    let nta = *NTA.lock().expect("unable to get saved nta");
    for pocket in pockets.iter() {
        let elm_id = format!("pocket-{}", pocket.seat_idx);
        let elm = doc
            .get_element_by_id(&elm_id)
            .expect("could not find pocket");
        pocket.fill_element(&elm);
        if pocket.seat_idx == nta {
            elm.class_list().add_1("next-action").unwrap();
        } else {
            elm.class_list().remove_1("next-action").unwrap();
        }
    }
}

fn redraw_community() {
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id("community").unwrap();
    let community = COMMUNITY.lock().expect("unable to get saved community");
    let comm: Vec<Card> = community
        .iter()
        .take_while(|c| c.is_some())
        .map(|c| c.unwrap())
        .collect();
    Community(comm).fill_element(&elm);
}

fn redraw_pot() {
    let pot = POT.lock().expect("unable to get saved pot");
    let doc = web_sys::window()
        .expect("No window?")
        .document()
        .expect("No document?");
    let elm = doc.get_element_by_id("pot").unwrap();
    Pot(pot.to_vec()).fill_element(&elm);
}

/// Redraw the table/hands/etc. based on the given state object. Return the number of seconds we
/// should wait before polling for a new update and the last sequence number we observed.
#[wasm_bindgen]
pub fn redraw(changes_message_str: String) -> i32 {
    let changes_message: Msg = serde_json::from_str(&changes_message_str).unwrap();
    let logs = match changes_message {
        Msg::GameLogs(logs) => logs,
        _ => {
            log("redraw given msg that isn't game logs");
            return 2;
        }
    };
    let mut need_redraw_pockets = false;
    let mut need_redraw_action_buttons = false;
    let mut need_redraw_community = false;
    let mut need_redraw_pot = false;
    let mut saved_logs = SAVED_LOGS.lock().expect("could not get saved logs");
    saved_logs.extend(logs.iter().cloned());
    for (idx, item) in logs.iter() {
        log(&format!("{idx}: {:?}", item));
        match item {
            LogItem::NewBaseState(bs) => {
                let mut pockets = POCKETS.lock().expect("could not get saved pockets");
                let pi_cache = PLAYER_INFO
                    .lock()
                    .expect("could not lock player info cache");
                pockets.clear();
                for (seat_idx, player) in bs
                    .seats
                    .iter()
                    .enumerate()
                    .filter(|(_, seat)| seat.is_some())
                    .map(|(idx, seat)| (idx, seat.unwrap()))
                {
                    let name = pi_cache
                        .get(&player.id)
                        .map(|pi| pi.username.clone())
                        .unwrap_or_else(|| format!("Player {}", player.id));
                    let pocket = Pocket {
                        cards: None,
                        name,
                        stack: player.stack,
                        seat_idx,
                        player_id: player.id,
                        bet_status: BetStatus::Waiting,
                        is_btn: false,
                        is_sb: false,
                        is_bb: false,
                    };
                    pockets.push(pocket);
                    need_redraw_pockets = true;
                }
                *COMMUNITY.lock().expect("unable to get saved community") = [None; 5];
                need_redraw_community = true;
                need_redraw_action_buttons = true;
                need_redraw_pot = true;
            }
            LogItem::PocketDealt(player_id, cards) => {
                let mut pockets = POCKETS.lock().expect("could not get saved pockets");
                for pocket in pockets.iter_mut() {
                    if pocket.player_id == *player_id {
                        pocket.cards = Some(match cards {
                            None => [None, None],
                            Some(cards) => [Some(cards[0]), Some(cards[1])],
                        });
                        need_redraw_pockets = true;
                    }
                }
            }
            LogItem::TokensSet(btn, sb, bb) => {
                let mut pockets = POCKETS.lock().expect("could not get saved pockets");
                for pocket in pockets.iter_mut() {
                    if pocket.seat_idx == *btn {
                        pocket.is_btn = true;
                        need_redraw_pockets = true;
                    }
                    if pocket.seat_idx == *sb {
                        pocket.is_sb = true;
                        need_redraw_pockets = true;
                    }
                    if pocket.seat_idx == *bb {
                        pocket.is_bb = true;
                        need_redraw_pockets = true;
                    }
                }
            }
            LogItem::NextToAct(seat) => {
                *NTA.lock().expect("could not get saved nta") = *seat;
                need_redraw_action_buttons = true;
            }
            LogItem::Pot(pot_item) => match pot_item {
                pot::LogItem::Bet(player_id, bet_action) => {
                    let bet_status: BetStatus = (*bet_action).into();
                    let mut pockets = POCKETS.lock().expect("could not get saved pockets");
                    for pocket in pockets.iter_mut() {
                        if pocket.player_id == *player_id {
                            let old_bet_status = pocket.bet_status;
                            pocket.bet_status = bet_status;
                            need_redraw_pockets = true;
                            let old_wager = match old_bet_status {
                                BetStatus::In(x) | BetStatus::AllIn(x) => x,
                                BetStatus::Folded | BetStatus::Waiting => 0,
                            };
                            match bet_status {
                                BetStatus::In(new_wager) | BetStatus::AllIn(new_wager) => {
                                    pocket.stack += old_wager;
                                    pocket.stack -= new_wager;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                pot::LogItem::RoundEnd(_)
                | pot::LogItem::EntireStakeInPot(_, _, _)
                | pot::LogItem::PartialStakeInPot(_, _, _, _)
                | pot::LogItem::NewPotCreated(_, _, _) => {}
                pot::LogItem::Payouts(_, _) => {}
                pot::LogItem::BetsSorted(v) => {
                    let mut pot = POT.lock().expect("unable to get saved pot");
                    for (_player_id, stake) in v.iter() {
                        if pot.is_empty() {
                            pot.push(0);
                        }
                        pot[0] += stake.amount;
                    }
                    need_redraw_pot = true;
                }
            },
            LogItem::CurrentBetSet(_, cb, _, mr) => {
                *CURRENT_BET_AND_RAISE
                    .lock()
                    .expect("unable to get current bet and min raise") = (*cb, *mr);
            }
            LogItem::StateChange(old, new) => {
                if old == new {
                    continue;
                }
                let mut pockets = POCKETS.lock().expect("unable to get saved pockets");
                for pocket in pockets.iter_mut() {
                    pocket.bet_status = BetStatus::Waiting;
                }
                need_redraw_pockets = true;
            }
            LogItem::Flop(c1, c2, c3) => {
                let mut comm = COMMUNITY.lock().expect("unable to get saved community");
                comm[0] = Some(*c1);
                comm[1] = Some(*c2);
                comm[2] = Some(*c3);
                need_redraw_community = true;
            }
            LogItem::Turn(c) => {
                let mut comm = COMMUNITY.lock().expect("unable to get saved community");
                comm[3] = Some(*c);
                need_redraw_community = true;
            }
            LogItem::River(c) => {
                let mut comm = COMMUNITY.lock().expect("unable to get saved community");
                comm[4] = Some(*c);
                need_redraw_community = true;
            }
        }
    }
    if need_redraw_pockets {
        redraw_pockets();
    }
    if need_redraw_action_buttons {
        redraw_action_buttons(is_self_nta());
    }
    if need_redraw_community {
        redraw_community();
    }
    if need_redraw_pot {
        redraw_pot();
    }
    //let state: FilteredGameState = serde_json::from_str(&state).unwrap();
    //let mut last_state = LAST_STATE.lock().expect("could not get last state");
    //if last_state.is_some() && *last_state.as_ref().unwrap() == state {
    //    return if is_self_nta(&state) { 30 } else { 2 };
    //}
    //*last_state = Some(state.clone());
    //send_player_info_requests_for_missing_players(&state);
    //redraw_table(&state);
    //redraw_logs(&state.logs);
    //redraw_state(&state);
    //redraw_action_buttons(&state);
    if is_self_nta() {
        30
    } else {
        2
    }
}

#[wasm_bindgen]
pub fn get_last_seq_num() -> SeqNum {
    let logs = SAVED_LOGS.lock().expect("unable to get saved logs");
    if logs.is_empty() {
        0
    } else {
        logs[logs.len() - 1].0
    }
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

fn last_seq_num() -> SeqNum {
    let logs = SAVED_LOGS.lock().expect("unable to get saved logs");
    if logs.is_empty() {
        0
    } else {
        logs[logs.len() - 1].0
    }
}

#[wasm_bindgen]
pub fn onclick_fold() {
    let msg = Msg::Action(action::Msg::Fold);
    send_action(last_seq_num(), &serde_json::to_string(&msg).unwrap());
}

#[wasm_bindgen]
pub fn onclick_call() {
    let msg = Msg::Action(action::Msg::Call);
    send_action(last_seq_num(), &serde_json::to_string(&msg).unwrap());
}

#[wasm_bindgen]
pub fn onclick_check() {
    let msg = Msg::Action(action::Msg::Check);
    send_action(last_seq_num(), &serde_json::to_string(&msg).unwrap());
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
    send_action(last_seq_num(), &serde_json::to_string(&msg).unwrap());
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
    send_action(last_seq_num(), &serde_json::to_string(&msg).unwrap());
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

#[wasm_bindgen]
pub fn save_player_info(pi: String) {
    let info: PlayerInfo =
        serde_json::from_str(&pi).expect("Unable to deserialize PlayerInfo json");
    let mut cache = PLAYER_INFO
        .lock()
        .expect("could not lock player info cache");
    log(&format!("Got player info {}: {:?}", info.id, info));
    cache.insert(info.id, info);
}
