use crate::Currency;
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

#[derive(Debug, derive_more::Display, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum BetStatus {
    Folded,
    Waiting,
    In(Currency),
    AllIn(Currency),
}

impl Default for BetStatus {
    fn default() -> Self {
        BetStatus::Waiting
    }
}

impl From<BetAction> for BetStatus {
    fn from(ba: BetAction) -> Self {
        match ba {
            BetAction::AllIn(x) => BetStatus::AllIn(x),
            BetAction::Fold => BetStatus::Folded,
            BetAction::Bet(x) | BetAction::Call(x) | BetAction::Raise(x) => BetStatus::In(x),
            BetAction::Check => BetStatus::In(0),
        }
    }
}
