pub mod bet;
pub mod cards;
pub mod log;
pub mod player;
pub mod pot;
pub mod state;
mod util;

pub use cards::{deck, hand};

pub const MAX_PLAYERS: usize = 12;
pub type PlayerId = i32;
pub type Currency = i32;
pub type SeqNum = usize;
pub type SeatIdx = usize;

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum GameError {
    PlayerAlreadySeated,
    TableFull,
    NotEnoughPlayers,
    StreetNotComplete,
    PlayerNotFound,
    PlayerIsNotBetting,
    NoBetExpected,
    OutOfTurn,
    PlayerStackTooShort,
    InvalidBet,
}
