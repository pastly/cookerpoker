//! Client --> Server messages for fold, call, etc.

use poker_core::Currency;
use serde::{Deserialize, Serialize};

/// Wrapper for all our types of messages to help de/serialize
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Msg {
    Fold,
    Call,
    Check,
    Bet(Currency),
    Raise(Currency),
}
