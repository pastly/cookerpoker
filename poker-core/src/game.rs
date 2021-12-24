pub mod players;
pub mod pot;
pub mod table;

pub use super::{deck, hand};
pub use players::PlayerId;
pub use pot::Currency;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BetAction {
    Check,
    Fold,
    Call(Currency),
    Bet(Currency),
    Raise(Currency),
    AllIn(Currency),
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
    BetLowerThanCall,
    InvalidCall,
    PlayerIsNotBetting,
    PlayerNotFound,
    BadAction,
}

impl std::error::Error for BetError {}

#[derive(Debug, derive_more::Display)]
pub enum GameError {
    DeckError(deck::DeckError),
    BetError(BetError),
    NotEnoughPlayers,
    SeatTaken,
    PlayerAlreadySeated,
    InvalidSeat,
    BettingPlayerCantStand,
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
