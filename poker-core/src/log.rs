use crate::deck::Card;
use crate::pot;
use crate::state;
use crate::{Currency, PlayerId, SeqNum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogItem {
    Pot(pot::LogItem),
    NewBaseState(Box<state::BaseState>),
    StateChange(state::State, state::State),
    TokensSet(usize, usize, usize), // btn/sb/bb seat indexes into player array
    NextToAct(usize),               // seat index into player array
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
            LogItem::NewBaseState(bs) => write!(f, "{bs}"),
            LogItem::TokensSet(btn, sb, bb) => write!(f, "BTN/SB/BB set to seats {btn}/{sb}/{bb}"),
            LogItem::NextToAct(idx) => write!(f, "Next to act is seat {idx}"),
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

#[derive(Debug, PartialEq, Eq, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Log {
    active: Vec<(SeqNum, LogItem)>,
    archive: Vec<(SeqNum, LogItem)>,
    last_seq_num: SeqNum,
}

impl Log {
    pub(crate) fn push(&mut self, item: LogItem) {
        let seq = self.last_seq_num + 1;
        self.active.push((seq, item));
        self.last_seq_num = seq;
    }

    pub(crate) fn extend<I: IntoIterator<Item = LogItem>>(&mut self, iter: I) {
        let start = self.last_seq_num + 1;
        for (seq, item) in (start..).zip(iter) {
            self.active.push((seq, item));
            self.last_seq_num = seq;
        }
    }

    pub(crate) fn clear(&mut self) {
        self.archive.append(&mut self.active);
    }

    pub(crate) fn items_since(
        &self,
        oldest_seq: SeqNum,
    ) -> impl Iterator<Item = (SeqNum, LogItem)> + '_ {
        let iter1 = self
            .archive
            .iter()
            .skip_while(move |(seq, _item)| *seq <= oldest_seq)
            .cloned();
        let iter2 = self
            .active
            .iter()
            .skip_while(move |(seq, _item)| *seq <= oldest_seq)
            .cloned();
        iter1.chain(iter2)
    }
}
