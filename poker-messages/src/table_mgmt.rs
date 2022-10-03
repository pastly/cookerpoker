//! Client <--> Server messages that aren't core to a poker hand, such as people
//! sitting down/standing up.

//use poker_core::game::{Currency, PlayerId};
use serde::{Deserialize, Serialize};
use std::fmt;

type TableId = i32;

/// Wrapper for all our types of messages to help de/serialize
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Msg {
    SitIntent(SitIntent),
    SitIntentResp(SitIntentResp),
}

/// Error codes for all server -> client messages
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RespErrCode {
    // TODO: Check the size of Option<RespErrCode> and if larger than RespErrCode,
    // fix that.
    NoOpenSeat,
    //NotEnoughMoney,
}

impl fmt::Display for RespErrCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::NoOpenSeat => "No open seat",
            }
        )
    }
}

/// Client --> Server: A player intends to sit down at a table. They may not be
/// allowed to for some reason.
///
/// The player is implicit from the authenticated user that is sending the message
/// Starting stack info may need to be added.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitIntent {
    table_id: TableId,
}

impl SitIntent {
    pub fn new(table_id: TableId) -> Self {
        Self { table_id }
    }
}

/// Server --> Client: Whether or not the given SitIntent is accepted AKA
/// whether the player has sat down.
///
/// Client should expect a follow up message indicating their seat as well as
/// everyone else's.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitIntentResp {
    sit_intent: SitIntent,
    error: Option<RespErrCode>,
}
