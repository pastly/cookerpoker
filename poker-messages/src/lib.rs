pub mod action;

use poker_core::log::LogItem;
use poker_core::SeqNum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Msg {
    Action(action::Msg),
    GameLogs(Vec<(SeqNum, LogItem)>),
}
