use poker_core::PlayerId;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub(crate) struct PlayerInfo {
    pub(crate) id: PlayerId,
    pub(crate) username: String,
}
