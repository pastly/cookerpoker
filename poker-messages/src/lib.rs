use poker_core::deck::Card;
use poker_core::game::{Currency, PlayerId};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub type SeqNum = u32;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionList(pub Vec<Action>);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub seq: SeqNum,
    pub action: ActionEnum,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionEnum {
    SitDown(SitDown),
    StandUp(StandUp),
    CardsDealt(CardsDealt),
    CommunityDealt(CommunityDealt),
    Epoch(Epoch),
    Reveal(Reveal),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub player_id: PlayerId,
    pub name: String,
    pub monies: Currency,
    pub seat: usize,
}

impl PlayerInfo {
    pub fn new<P: Into<PlayerId>, C: Into<Currency>>(
        player_id: P,
        name: String,
        monies: C,
        seat: usize,
    ) -> Self {
        Self {
            player_id: player_id.into(),
            name,
            monies: monies.into(),
            seat,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitDown {
    player_info: PlayerInfo,
}

impl SitDown {
    pub fn new(player_info: PlayerInfo) -> Self {
        Self { player_info }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandUp {
    seat: usize,
}

impl StandUp {
    pub fn new(seat: usize) -> Self {
        Self { seat }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardsDealt {
    pub seats: Vec<usize>,
    pub pocket: [Card; 2],
}

impl CardsDealt {
    pub fn new(seats: Vec<usize>, pocket: [Card; 2]) -> Self {
        Self { seats, pocket }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunityDealt {
    cards: Vec<Card>,
}

impl CommunityDealt {
    pub fn new(cards: Vec<Card>) -> Self {
        Self { cards }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Epoch {
    pub players: Vec<PlayerInfo>,
    pub blinds: (Currency, Currency),   // small, big
    pub buttons: (usize, usize, usize), // seat indexes of dealer, small blind, dealer blind
    pub decision_time: Duration,
}

impl Epoch {
    pub fn new<C: Into<Currency>>(
        players: Vec<PlayerInfo>,
        blinds: (C, C),
        buttons: (usize, usize, usize),
        decision_time: Duration,
    ) -> Self {
        Self {
            players,
            blinds: (blinds.0.into(), blinds.1.into()),
            buttons,
            decision_time,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reveal {
    pub seat: usize,
    pub pocket: [Card; 2],
}

impl Reveal {
    pub fn new(seat: usize, pocket: [Card; 2]) -> Self {
        Self { seat, pocket }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// More of just a demonstration of how to use these messages than an actual test. We should
    /// assume serde can serialize/deserialize correctly.
    #[test]
    fn demonstrate_usage() {
        let a = Action {
            seq: 1,
            action: ActionEnum::SitDown(SitDown::new(PlayerInfo::new(
                10,
                "Mutt".to_string(),
                100,
                0,
            ))),
        };
        let s = serde_json::to_string(&a).unwrap();
        let b = serde_json::from_str(&s).unwrap();
        assert_eq!(a, b);
        println!("{}", s);
    }
}
