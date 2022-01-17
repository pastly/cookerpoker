use super::deck::Card;
use super::pot;
use super::table::GameState;
use super::{Currency, PlayerId};
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug)]
pub enum LogItem {
    Pot(pot::LogItem),
    StateChange(GameState),
    PocketsDealt(HashMap<PlayerId, Option<[Card; 2]>>),
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
            LogItem::PocketsDealt(map) => {
                let middle: String = map
                    .iter()
                    .map(|(player, p)| {
                        format!(
                            "p{}: {}",
                            player,
                            match p {
                                None => "????".to_string(),
                                Some(p) => format!("{}{}", p[0], p[1]),
                            }
                        )
                    })
                    .join(", ");
                let s = "[".to_string() + &middle + "]";
                write!(f, "Pockets dealt: {}", s)
            }
            LogItem::SitDown(p, seat, monies) => {
                write!(f, "p{} sits in seat {} with {}", p, seat, monies)
            }
            LogItem::StandUp(p, monies) => write!(f, "p{} leaves the table with {}", p, monies),
            LogItem::CurrentBetSet(x, y) => write!(f, "Current bet to match is now {}; minimum raise is now {}", x, y),
            LogItem::Flop(c) => write!(f, "Flop: {} {} {}", c[0], c[1], c[2]),
            LogItem::Turn(c) => write!(f, "Turn: {}", c),
            LogItem::River(c) => write!(f, "River: {}", c),
        }
    }
}

/// Given a set of [`LogItem']s, filter them down in some way.
pub trait LogFilter {
    fn filter(&self, logs: Vec<LogItem>) -> Vec<LogItem>;
}

/// A [`LogFilter`] that doesn't filter anything out.
pub struct AllLogsFilter;

impl LogFilter for AllLogsFilter {
    fn filter(&self, logs: Vec<LogItem>) -> Vec<LogItem> {
        logs
    }
}

/// A [`LogFilter`] that only keeps logs necessary for a player client to play the
/// game. This censors logs that contain sensitive information, namely other players'
/// pockets.
///
/// See [`PlayerVerboseFilter`] for a still-censored version of this that includes all
/// non-sensitive information too (e.g. pot calculation information.)
pub struct PlayerFilter(pub PlayerId);

impl LogFilter for PlayerFilter {
    fn filter(&self, logs: Vec<LogItem>) -> Vec<LogItem> {
        let mut logs = PlayerVerboseFilter(self.0).filter(logs);
        logs.retain(|log| match log {
            LogItem::Pot(ref pli) => match pli {
                // Keep these, as these are how players find out about betting.
                pot::LogItem::Bet(_, _) => true,
                // Pot will log sub-pot payouts. These are denoated by Some(pot_n). None here
                // means it's total payout, which we should keep.
                pot::LogItem::Payouts(pot_n, _) => pot_n.is_none(),
                pot::LogItem::RoundEnd(_)
                | pot::LogItem::BetsSorted(_)
                | pot::LogItem::EntireStakeInPot(_, _, _)
                | pot::LogItem::PartialStakeInPot(_, _, _, _)
                | pot::LogItem::NewPotCreated(_, _, _) => false,
            },
            // These are how players find out about new round (?) and what table cards came
            LogItem::Flop(_) | LogItem::Turn(_) | LogItem::River(_) => true,
            // PlayerVerboseFilter should have filtered down to 1 (or 0, I guess) pocket here,
            // which should match our player id. Assert that this is the case. If it isn't the
            // case, there's something wrong in PlayerVerboseFilter, and the fix should be made
            // there.
            LogItem::PocketsDealt(ref map) => {
                let n_some = map.values().filter(|v| v.is_some()).count();
                assert!(n_some <= 1);
                true
            }
            // We may determine some of these are needed in the future. I'm thinking
            // SitDown/StandUp namely. For now, filter out.
            LogItem::StateChange(_)
            | LogItem::SitDown(_, _, _)
            | LogItem::StandUp(_, _)
            | LogItem::CurrentBetSet(_, _) => false,
        });
        logs
    }
}

/// A [`LogFilter`] that keeps everything except sensitive information that the given [`PlayerId`]
/// should not be allowed to see (i.e. other players' pockets).
///
/// The given PlayerId can be any PlayerId (the player doesn't have to be in the game).
pub struct PlayerVerboseFilter(pub PlayerId);

impl LogFilter for PlayerVerboseFilter {
    /// Rather straight forward filtering. Keep everything, but edit the pockets hashmap to only
    /// include the given player's pocket.
    fn filter(&self, mut logs: Vec<LogItem>) -> Vec<LogItem> {
        for log in &mut logs {
            if let LogItem::PocketsDealt(map) = log {
                for (&k, v) in map.iter_mut() {
                    if k != self.0 {
                        *v = None;
                    }
                }
            }
        }
        logs
    }
}

/// Like [`PlayerFilter`], but no per-player sensitive information is kept.
pub struct SpectatorFilter;

impl LogFilter for SpectatorFilter {
    /// Straight forward filtering like [`PlayerFilter`]. Since we want to be just like it,
    /// but with no eyes on per-player sensitive info, just use PlayerFilter directly and
    /// remove the last bit of per-player sensitive info.
    fn filter(&self, logs: Vec<LogItem>) -> Vec<LogItem> {
        // An arbitrary player id so we can construct a PlayerFilter. We'll filter out
        // per-player sensitive info that somehow might exist for this id momentarily.
        let mut logs = PlayerFilter(42069).filter(logs);
        for log in &mut logs {
            if let LogItem::PocketsDealt(map) = log {
                for v in map.values_mut() {
                    *v = None;
                }
            }
        }
        logs
    }
}

/// Like [`PlayerVerboseFilter`], but no per-player sensitive information is kept.
pub struct SpectatorVerboseFilter;

impl LogFilter for SpectatorVerboseFilter {
    /// Straight forward filtering like [`PlayerVerboseFilter`]. Since we want to be just like it,
    /// but with no eyes on per-player sensitive info, just use PlayerVerboseFilter directly and
    /// remove the last bit of per-player sensitive info.
    fn filter(&self, logs: Vec<LogItem>) -> Vec<LogItem> {
        // An arbitrary player id so we can construct a PlayerVerboseFilter. We'll filter out
        // per-player sensitive info that somehow might exist for this id momentarily.
        let mut logs = PlayerVerboseFilter(42069).filter(logs);
        for log in &mut logs {
            if let LogItem::PocketsDealt(map) = log {
                for v in map.values_mut() {
                    *v = None;
                }
            }
        }
        logs
    }
}
