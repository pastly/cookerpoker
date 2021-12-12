use crate::elements::Elementable;
use poker_core::{deck::Card, PlayerId};
use poker_messages::*;
use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::Element;

#[derive(Debug)]
pub(crate) enum RenderError {
    NoPlayerInSeat(usize),
    NoPlayerWithId(PlayerId),
    JsHtmlError(JsValue),
}

impl std::error::Error for RenderError {}

impl From<JsValue> for RenderError {
    fn from(js: JsValue) -> Self {
        Self::JsHtmlError(js)
    }
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsHtmlError(e) => write!(f, "JS/HTML error: {:?}", e),
            Self::NoPlayerInSeat(n) => write!(
                f,
                "Asked to find PlayerInfo in seat {}, but no known player there.",
                n
            ),
            Self::NoPlayerWithId(id) => write!(
                f,
                "Asked to find PlayerInfo for id={}, but no known player",
                id
            ),
        }
    }
}

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

fn table_header() -> Result<Element, RenderError> {
    let tr = base_element("tr");
    let mut th = base_element("th");
    th.set_text_content(Some(&"Seq Num".to_string()));
    tr.append_child(&th)?;

    th = base_element("th");
    th.set_text_content(Some(&"Action".to_string()));
    tr.append_child(&th)?;

    th = base_element("th");
    th.set_text_content(Some(&"Pocket Cards".to_string()));
    tr.append_child(&th)?;

    Ok(tr)
}

fn pocket_cell_class(seat: usize) -> String {
    format!("gamelog-seat-{}-pocket", seat)
}

fn fill_pockets(
    table: &Element,
    pocket_map: &HashMap<usize, [Card; 2]>,
) -> Result<(), RenderError> {
    for (seat, pocket) in pocket_map.iter() {
        let class = pocket_cell_class(*seat);
        let elms = table.get_elements_by_class_name(&class);
        for elm in (0..elms.length()).map(|n| elms.item(n).unwrap()) {
            while let Some(child) = elm.last_child() {
                elm.remove_child(&child)?;
            }
            elm.append_child(&Some(pocket[0]).into_element())?;
            elm.append_child(&Some(pocket[1]).into_element())?;
        }
    }
    Ok(())
}

fn player_desc(p: &PlayerInfo) -> String {
    format!("{} (seat {})", p.name, p.seat)
}

fn player_from_id(e: &Epoch, id: PlayerId) -> Result<&PlayerInfo, RenderError> {
    for player_info in &e.players {
        if player_info.player_id == id {
            return Ok(player_info);
        }
    }
    Err(RenderError::NoPlayerWithId(id))
}

fn player_from_seat(e: &Epoch, seat: usize) -> Result<&PlayerInfo, RenderError> {
    for player_info in &e.players {
        if player_info.seat == seat {
            return Ok(player_info);
        }
    }
    Err(RenderError::NoPlayerInSeat(seat))
}

fn add_row_reveal(table: &Element, e: &Epoch, r: &Reveal, seq: SeqNum) -> Result<(), RenderError> {
    let tr = base_element("tr");
    let mut td = base_element("td");
    td.set_text_content(Some(&seq.to_string()));
    tr.append_child(&td)?;

    td = base_element("td");
    let p = player_from_seat(e, r.seat)?;
    let s = format!("{} reveals pocket.", player_desc(p));
    td.set_text_content(Some(&s));
    tr.append_child(&td)?;

    td = base_element("td");
    td.set_class_name(&pocket_cell_class(r.seat));
    td.append_child(&None.into_element())?;
    td.append_child(&None.into_element())?;
    tr.append_child(&td)?;

    table.append_child(&tr)?;
    Ok(())
}

fn add_row_cards_dealt(
    table: &Element,
    e: &Epoch,
    cd: &CardsDealt,
    seq: SeqNum,
) -> Result<(), RenderError> {
    for seat in &cd.seats {
        println!("{}", seat);
        let tr = base_element("tr");
        let mut td = base_element("td");
        td.set_text_content(Some(&seq.to_string()));
        tr.append_child(&td)?;

        td = base_element("td");
        let p = player_from_seat(e, *seat)?;
        let s = format!("{} receives cards.", player_desc(p));
        td.set_text_content(Some(&s));
        tr.append_child(&td)?;

        td = base_element("td");
        td.set_class_name(&pocket_cell_class(*seat));
        td.append_child(&None.into_element())?;
        td.append_child(&None.into_element())?;
        tr.append_child(&td)?;

        table.append_child(&tr)?;
    }
    Ok(())
}

pub(crate) fn render_html_list(
    log: &ActionList,
    root: &Element,
    self_player_id: PlayerId,
) -> Result<(), RenderError> {
    while let Some(child) = root.last_child() {
        root.remove_child(&child)?;
    }
    let table_header = table_header()?;
    let mut pocket_map: HashMap<usize, [Card; 2]> = HashMap::new();
    let mut table = base_element("table");
    table.append_child(&table_header)?;
    let mut last_epoch: Option<&Epoch> = None;
    for seq_action in &log.0 {
        let (seq, action) = (seq_action.seq, &seq_action.action);
        match action {
            ActionEnum::Epoch(a) => {
                if last_epoch.is_some() {
                    fill_pockets(&table, &pocket_map)?;
                    root.append_child(&table)?;
                    table = base_element("table");
                    table.append_child(&table_header)?;
                }
                pocket_map.clear();
                last_epoch = Some(a);
            }
            ActionEnum::CardsDealt(cd) => {
                add_row_cards_dealt(&table, last_epoch.unwrap(), cd, seq)?;
                pocket_map.insert(
                    player_from_id(last_epoch.unwrap(), self_player_id)?.seat,
                    cd.pocket,
                );
            }
            ActionEnum::Reveal(r) => {
                add_row_reveal(&table, last_epoch.unwrap(), r, seq)?;
                pocket_map.insert(r.seat, r.pocket);
            }
            _ => unimplemented!(),
        }
    }
    fill_pockets(&table, &pocket_map)?;
    root.append_child(&table)?;
    Ok(())
}
