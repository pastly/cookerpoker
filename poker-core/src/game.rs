pub mod players;
pub mod pot;
pub mod table;

pub use super::{deck, hand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum BetAction {
    Check,
    Fold,
    Call(i32),
    Bet(i32),
    AllIn(i32),
}

#[derive(Debug)]
pub enum BetError {
    AllInWithoutBeingAllIn,
    HasNoMoney,
    BetLowerThanCall,
    InvalidCall,
    PlayerIsNotBetting,
    PlayerNotFound,
    BadAction,
}

#[derive(Debug)]
pub enum GameError {
    DeckError(deck::DeckError),
    BetError(BetError),
    NotEnoughPlayers,
    SeatTaken,
    PlayerAlreadySeated,
    InvalidSeat,
}

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
