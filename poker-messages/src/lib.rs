pub mod action;

use poker_core::PlayerId;
use serde::{Deserialize, Serialize};

pub type SeqNum = u32;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Msg {
    Action(action::Msg),
    PlayerAction(PlayerId, action::Msg),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerialMsg(SeqNum, Msg);
