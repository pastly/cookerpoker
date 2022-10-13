use super::deck::Card;
use super::pot;
use super::table::GameState;
use super::{Currency, PlayerId};

#[derive(Debug, Clone)]
pub enum LogItem {
    Pot(pot::LogItem),
    StateChange(GameState),
    PocketDealt(PlayerId, [Card; 2]),
    SitDown(PlayerId, usize, Currency),
    StandUp(PlayerId, Currency),
    CurrentBetSet(Currency, Currency),
    Flop([Card; 3]),
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
            LogItem::StateChange(to) => write!(f, "State changed to {to}"),
            LogItem::PocketDealt(player, pocket) => {
                write!(f, "p{player} dealt {}{}", pocket[0], pocket[1])
            }
            LogItem::SitDown(p, seat, monies) => {
                write!(f, "p{} sits in seat {} with {}", p, seat, monies)
            }
            LogItem::StandUp(p, monies) => write!(f, "p{} leaves the table with {}", p, monies),
            LogItem::CurrentBetSet(x, y) => write!(
                f,
                "Current bet to match is now {}; minimum raise is now {}",
                x, y
            ),
            LogItem::Flop(c) => write!(f, "Flop: {} {} {}", c[0], c[1], c[2]),
            LogItem::Turn(c) => write!(f, "Turn: {}", c),
            LogItem::River(c) => write!(f, "River: {}", c),
        }
    }
}
