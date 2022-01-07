pub mod players;
pub mod pot;
pub mod table;

use self::hand::HandError;

pub use super::{deck, hand};
use itertools::Itertools;
pub use players::PlayerId;
pub use pot::Currency;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use table::GameState;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BetAction {
    Check,
    Fold,
    Call(Currency),
    Bet(Currency),
    Raise(Currency),
    AllIn(Currency),
}

impl BetAction {
    pub const fn is_allin(&self) -> bool {
        matches!(self, &BetAction::AllIn(_))
    }
}

impl std::fmt::Display for BetAction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BetAction::Check => write!(f, "Check"),
            BetAction::Fold => write!(f, "Fold"),
            BetAction::Call(v) => write!(f, "Call({})", v),
            BetAction::Bet(v) => write!(f, "Bet({})", v),
            BetAction::Raise(v) => write!(f, "Raise({})", v),
            BetAction::AllIn(v) => write!(f, "AllIn({})", v),
        }
    }
}

#[derive(Debug, derive_more::Display)]
pub enum BetError {
    AllInWithoutBeingAllIn,
    HasNoMoney,
    BetTooLow,
    BetTooHigh,
    PlayerIsNotBetting,
    PlayerNotFound,
    CantRaiseSelf,
    BadAction,
    OutOfTurn,
    NoBetExpected,
}

impl std::error::Error for BetError {}

#[derive(Debug, derive_more::Display)]
pub enum GameError {
    DeckError(deck::DeckError),
    BetError(BetError),
    HandError(HandError),
    NotEnoughPlayers,
    SeatTaken,
    PlayerAlreadySeated,
    InvalidSeat,
    BettingPlayerCantStand,
    BetNotExpected,
    RoundNotOver,
    InvalidBet(String),
}

impl std::error::Error for GameError {}

impl From<deck::DeckError> for GameError {
    fn from(d: deck::DeckError) -> Self {
        GameError::DeckError(d)
    }
}

impl From<BetError> for GameError {
    fn from(d: BetError) -> Self {
        GameError::BetError(d)
    }
}

impl From<HandError> for GameError {
    fn from(d: HandError) -> Self {
        GameError::HandError(d)
    }
}

#[derive(Debug)]
pub enum LogItem {
    Pot(pot::LogItem),
    StateChange(GameState),
    NewDeck(deck::DeckSeed),
    PocketsDealt(HashMap<PlayerId, [deck::Card; 2]>),
}

impl From<pot::LogItem> for LogItem {
    fn from(i: pot::LogItem) -> Self {
        Self::Pot(i)
    }
}

impl std::fmt::Display for LogItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogItem::Pot(pli) => write!(f, "{}", pli.to_string()),
            LogItem::StateChange(to) => write!(f, "State changed to {}", to),
            LogItem::NewDeck(ds) => write!(f, "Using deck with seed {}", ds),
            LogItem::PocketsDealt(map) => {
                let middle: String = map
                    .iter()
                    .map(|(player, p)| format!("p{}: {}{}", player, p[0], p[1]))
                    .join(", ");
                let s = "[".to_string() + &middle + "]";
                write!(f, "Pockets dealt: {}", s)
            }
        }
    }
}
