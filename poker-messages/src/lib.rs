use poker_core::game::{Currency, PlayerId};
use serde::{Deserialize, Serialize};

pub type SeqNum = u32;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub seq: SeqNum,
    pub action: ActionEnum,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionEnum {
    SitDown(SitDown),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitDown {
    player_id: PlayerId,
    name: String,
    monies: Currency,
    seat_idx: usize,
}

impl SitDown {
    pub fn new<P: Into<PlayerId>, C: Into<Currency>>(
        player_id: P,
        name: String,
        monies: C,
        seat_idx: usize,
    ) -> Self {
        Self {
            player_id: player_id.into(),
            name,
            monies: monies.into(),
            seat_idx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// More of just a demonstration of how to use these messages than an actual test. We should
    /// assume serde can serialize/deserialize correctly.
    #[test]
    fn demonstrate_usage() {
        let a = Action{
            seq: 1,
            action: ActionEnum::SitDown(SitDown::new(10, "Mutt".to_string(), 100, 0)),
        };
        let s = serde_json::to_string(&a).unwrap();
        let b = serde_json::from_str(&s).unwrap();
        assert_eq!(a, b);
    }
}
