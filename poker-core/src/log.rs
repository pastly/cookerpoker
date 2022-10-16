use crate::deck::Card;
use crate::pot;
use crate::state;
use crate::{Currency, PlayerId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogItem {
    Pot(pot::LogItem),
    StateChange(state::State, state::State),
    CurrentBetSet(Currency, Currency, Currency, Currency),
    PocketDealt(PlayerId, Option<[Card; 2]>),
    Flop(Card, Card, Card),
    Turn(Card),
    River(Card),
}

impl From<pot::LogItem> for LogItem {
    fn from(i: pot::LogItem) -> Self {
        Self::Pot(i)
    }
}

impl std::fmt::Display for LogItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogItem::Pot(pli) => write!(f, "{pli}"),
            LogItem::StateChange(old, new) => write!(f, "State changed from {old} to {new}"),
            LogItem::CurrentBetSet(old_cb, new_cb, old_mr, new_mr) => {
                write!(f, "Current bet changed from {old_cb} to {new_cb}; min raise changed from {old_mr} to {new_mr}")
            }
            LogItem::PocketDealt(player_id, pocket) => match pocket {
                None => write!(f, "Player {player_id} dealt a hand"),
                Some(p) => write!(f, "Player {player_id} dealt {}{}", p[0], p[1]),
            },
            // LogItem::SitDown(p, seat, monies) => {
            //     write!(f, "p{} sits in seat {} with {}", p, seat, monies)
            // }
            // LogItem::StandUp(p, monies) => write!(f, "p{} leaves the table with {}", p, monies),
            LogItem::Flop(c1, c2, c3) => write!(f, "Flop: {c1} {c2} {c3}"),
            LogItem::Turn(c) => write!(f, "Turn: {c}"),
            LogItem::River(c) => write!(f, "River: {c}"),
        }
    }
}
