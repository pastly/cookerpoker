//! Client <--> Server messages useful during development and that should be removed before the
//! first proper release

use serde::{Deserialize, Serialize};
//use std::fmt;

/// Wrapper for all our types of messages to help de/serialize
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Msg {
    StartHand,
}
