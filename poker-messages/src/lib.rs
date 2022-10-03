pub mod game;
pub mod table_mgmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Msg {
    Mgmt(table_mgmt::Msg),
}

pub fn encode(msg: &Msg) -> String {
    serde_json::to_string(&msg).unwrap()
}

pub fn decode(s: &str) -> Msg {
    serde_json::from_str(s).expect("Unable to decode message from string")
}
