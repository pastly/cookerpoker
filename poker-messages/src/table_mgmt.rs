//! Client <--> Server messages that aren't core to a poker hand, such as people
//! sitting down/standing up.

use poker_core::game::{Currency, PlayerId};
use serde::{Deserialize, Serialize};
use std::fmt;

type TableId = i32;

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
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitIntent {
    player_id: PlayerId,
    table_id: TableId,
    monies: Currency,
}

impl SitIntent {
    pub fn new<C: Into<Currency>>(player_id: PlayerId, table_id: TableId, monies: C) -> Self {
        Self {
            player_id,
            table_id,
            monies: monies.into(),
        }
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
