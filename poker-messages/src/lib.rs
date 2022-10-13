pub mod action;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Msg {
    Action(action::Msg),
    // There will eventually be some other type of message, I think.
    SomethingElse,
}
