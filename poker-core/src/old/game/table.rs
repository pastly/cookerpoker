use crate::deck::DeckSeed;
use crate::game::{BetError, LogItem};
use crate::hand::best_hands;

use super::deck::{Card, Deck};
use super::players::{BetStatus, PlayerId, SeatedPlayers};
use super::pot::Pot;

use super::{BetAction, Currency, GameError};
use derive_more::Display;
use std::cmp::Ordering;
use std::mem;

const DEF_MAX_LIVE_HANDS: u16 = 2;
const DEF_MAX_ARCHIVE_HANDS: u16 = 100;

impl TableType {
    /// Helper function because dumb
    pub fn i(self) -> i16 {
        self.into()
    }
    pub const fn get_all_as_slice() -> [&'static str; 2] {
        ["Tournament", "Open"]
    }
    pub const fn get_error() -> &'static str {
        // TODO figure out how to do this from slice
        "Invalid TableType. Valid values are: Tournament, Open"
    }
}

#[derive(Debug, Clone, Copy, Display)]
pub enum TableType {
    Tournament,
    Open,
    Invalid,
}

impl From<i16> for TableType {
    fn from(f: i16) -> Self {
        match f {
            0 => Self::Tournament,
            1 => Self::Open,
            _ => Self::Invalid,
        }
    }
}

impl From<TableType> for i16 {
    fn from(tt: TableType) -> Self {
        match tt {
            TableType::Tournament => 0,
            TableType::Open => 1,
            TableType::Invalid => i16::MAX,
        }
    }
}

/// Public type representing a player's state in the current game and hand.
#[derive(Debug, Default)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub monies: Currency,
    pub bet_status: BetStatus,
    pub pocket: Option<[Card; 2]>,
    pub is_dealer: bool,
    pub is_small_blind: bool,
    pub is_big_blind: bool,
}

#[derive(Debug, Clone, Copy, derive_more::Display)]
pub enum GameState {
    NotStarted,
    Dealing,
    Betting(BetRound),
    Showdown,
    EndOfHand,
}

#[derive(Debug, Copy, Clone, derive_more::Display)]
pub enum BetRound {
    PreFlop,
    Flop,
    Turn,
    River,
}

impl Default for GameInProgress {
    fn default() -> Self {
        GameInProgress {
            table_type: TableType::Open,
            table_cards: [None; 5],
            seated_players: SeatedPlayers::default(),
            pot: Pot::default(),
            state: GameState::NotStarted,
            small_blind: 5.into(),
            current_bet: 10.into(),
            min_raise: 20.into(),
            last_raiser: None,
            hand_num: 0,
            deck: Deck::default(),
            live_logs: Vec::new(),
            archive_logs: Vec::new(),
            max_live_hands: DEF_MAX_LIVE_HANDS,
            max_archive_hands: DEF_MAX_ARCHIVE_HANDS,
        }
    }
}

#[derive(Debug)]
pub struct GameInProgress {
    pub table_type: TableType,
    pub table_cards: [Option<Card>; 5],
    seated_players: SeatedPlayers,
    pub pot: Pot,
    pub state: GameState,
    pub small_blind: Currency,
    /// The amount that each player is expected to match in order to make it to the end of the
    /// current betting round.
    pub current_bet: Currency,
    /// If a player wishes to raise this betting round, they must raise to at least this amount.
    /// This is the total amount to raise to, i.e. it is larger than current_bet.
    pub min_raise: Currency,
    /// The last person to raise this betting round.
    ///
    /// Needed because of the full bet rule. You can't raise, have action come back to you, then
    /// raise again without someone raising after your first raise. Action can come back to you
    /// like this if someone goes all in for less than the minimum raise after your first raise.
    ///
    /// It's confusing. See <https://duckduckgo.com/?t=ffab&q=allin+raise+less+than+minraise>
    last_raiser: Option<PlayerId>,
    pub hand_num: i16,
    deck: Deck,
    /// [`LogItem`]s for the current hand and the last couple game(s).
    live_logs: Vec<(usize, LogItem)>,
    /// [`LogItem`]s for older games
    archive_logs: Vec<(usize, LogItem)>,
    /// Maximum number of *hands* that live_logs should store.
    max_live_hands: u16,
    /// Maximum number of *hands* that archive_logs should store.
    max_archive_hands: u16,
}

impl GameInProgress {
    pub fn start_round(&mut self, seed: &DeckSeed) -> Result<(), GameError> {
        self.rotate_logs();
        let mut logs = vec![];
        self.state = GameState::Dealing;
        logs.push(LogItem::StateChange(self.state));
        self.deck = Deck::new(seed);
        self.table_cards = [None, None, None, None, None];

        // TODO save seed for DB, and perhaps the log
        self.hand_num += 1;

        // Handles auto folds and moving the okens
        self.seated_players.start_hand()?;

        // TODO log players in hand

        // Reset the pot
        self.pot = Pot::default();

        self.last_raiser = None;

        // Blinds bet
        let (small_blind, big_blind) = self
            .seated_players
            .blinds_bet(self.small_blind, self.big_blind())?;

        let mut pot_logs = vec![];
        pot_logs.append(&mut self.pot.bet(small_blind.0, small_blind.1));
        pot_logs.append(&mut self.pot.bet(big_blind.0, big_blind.1));
        logs.extend(pot_logs.into_iter().map(|l| l.into()));

        self.state = GameState::Betting(BetRound::PreFlop);
        logs.push(LogItem::StateChange(self.state));

        // Deal the pockets
        let nump = self.seated_players.betting_players_count() as u8;
        let pockets = self.deck.deal_pockets(nump)?;
        logs.extend(
            self.seated_players
                .deal_pockets(pockets)
                .into_iter()
                .map(|(k, v)| LogItem::PocketDealt(k, v)),
        );

        logs.push(LogItem::CurrentBetSet(self.current_bet, self.min_raise));
        self.logs_append(logs);
        Ok(())
    }

    /// Gets the seated player by id if they are seated at the current table.
    /// Front-end is responsible for making sure there isn't data leakage
    pub fn get_player_info(&self, player_id: PlayerId) -> Option<PlayerInfo> {
        let sp = match self.seated_players.player_by_id(player_id) {
            None => return None,
            Some(sp) => sp,
        };
        Some(PlayerInfo {
            id: sp.id,
            monies: sp.monies(),
            bet_status: sp.bet_status(),
            pocket: sp.pocket,
            is_dealer: self.seated_players.dealer_token == sp.seat_index,
            is_small_blind: self.seated_players.small_blind_token == sp.seat_index,
            is_big_blind: self.seated_players.big_blind_token == sp.seat_index,
        })
    }

    pub fn sit_down<C: Into<Currency> + Copy>(
        &mut self,
        player_id: PlayerId,
        monies: C,
        seat: usize,
    ) -> Result<(), GameError> {
        self.seated_players.sit_down(player_id, monies, seat)?;
        self.logs_push(LogItem::SitDown(player_id, seat, monies.into()));
        Ok(())
    }

    pub fn stand_up(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        let monies = match self.state {
            GameState::EndOfHand | GameState::NotStarted => self
                .seated_players
                .stand_up(player_id)
                .ok_or(GameError::UnknownPlayer)?,
            _ => {
                let p = self
                    .seated_players
                    .player_by_id(player_id)
                    .ok_or(GameError::UnknownPlayer)?;
                if p.is_betting() {
                    return Err(GameError::BettingPlayerCantStand);
                } else {
                    self.seated_players
                        .stand_up(player_id)
                        .ok_or(GameError::UnknownPlayer)?
                }
            }
        };
        self.logs_push(LogItem::StandUp(player_id, monies));
        Ok(())
    }

    /// The betting round has just ended. Advance to the next game state, and do intra-round
    /// bookkeeping, e.g. finialize the pot so far and reset the current bet amount.
    fn after_bet_advance_round(&mut self) -> Result<(GameState, Vec<LogItem>), GameError> {
        let mut logs = vec![];
        // determine next game state
        let next = match self.state {
            GameState::Betting(round) => match round {
                BetRound::PreFlop => GameState::Betting(BetRound::Flop),
                BetRound::Flop => GameState::Betting(BetRound::Turn),
                BetRound::Turn => GameState::Betting(BetRound::River),
                BetRound::River => GameState::Showdown,
            },
            _ => unreachable!(),
        };
        // bookkeeping
        self.seated_players.next_betting_round()?;
        let pot_logs = self.pot.finalize_round();
        self.current_bet = 0.into();
        self.min_raise = self.big_blind();
        self.last_raiser = None;
        logs.extend(pot_logs.into_iter().map(|pot_logitem| pot_logitem.into()));
        logs.push(LogItem::CurrentBetSet(self.current_bet, self.min_raise));
        // deal community cards, if needed
        if let GameState::Betting(round) = next {
            match round {
                BetRound::PreFlop => unreachable!(),
                BetRound::Flop => {
                    self.deck.burn();
                    let c1 = self.deck.draw()?;
                    let c2 = self.deck.draw()?;
                    let c3 = self.deck.draw()?;
                    self.table_cards[0] = Some(c1);
                    self.table_cards[1] = Some(c2);
                    self.table_cards[2] = Some(c3);
                    logs.push(LogItem::Flop([c1, c2, c3]));
                }
                BetRound::Turn => {
                    self.deck.burn();
                    let c1 = self.deck.draw()?;
                    self.table_cards[3] = Some(c1);
                    logs.push(LogItem::Turn(c1));
                }
                BetRound::River => {
                    self.deck.burn();
                    let c1 = self.deck.draw()?;
                    self.table_cards[4] = Some(c1);
                    logs.push(LogItem::River(c1));
                }
            }
        };
        Ok((next, logs))
    }

    /// Returns the PlayerId of the next player we expect a bet from, or None if we don't expect a
    /// bet from anyone at this time.
    pub fn next_player(&self) -> Option<PlayerId> {
        self.seated_players.next_player()
    }

    pub fn bet(&mut self, player: PlayerId, ba: BetAction) -> Result<(), GameError> {
        let mut logs = vec![];
        // Make sure we're in a state where bets are expected
        match self.state {
            GameState::Betting(_) => {}
            _ => return Err(GameError::BetNotExpected),
        }
        // Make sure bet is for an appropriate amount and no other errors are present
        match &ba {
            BetAction::Bet(x) | BetAction::Call(x) => {
                match x.cmp(&self.current_bet) {
                    Ordering::Less => return Err(BetError::BetTooLow.into()),
                    Ordering::Greater => {
                        // only an error if there is a non-zero current bet. It's 0 for the start of
                        // post-flop rounds
                        if self.current_bet != 0.into() {
                            return Err(BetError::BetTooHigh.into());
                        }
                    }
                    // No errors to account for and no maintenance to do
                    Ordering::Equal => {}
                }
            }
            // No errors to account for and no maintenance to do
            BetAction::Check | BetAction::Fold => {}
            // AllIn can be for any amount, so no errors to catch
            BetAction::AllIn(_) => {}
            BetAction::Raise(x) => {
                // A raise must meet the min raise
                if x < &self.min_raise {
                    return Err(BetError::BetTooLow.into());
                }
                // A player cannot raise if they were the most recent person to raise.
                if self.last_raiser.is_some() && self.last_raiser.unwrap() == player {
                    return Err(BetError::CantRaiseSelf.into());
                }
            }
        }

        if matches!(ba, BetAction::Raise(_)) {
            self.last_raiser = Some(player);
        }

        // Call seated players bet, which will convert to AllIn as neccesary
        let new_ba_and_current_bet = self.seated_players.bet(player, ba, self.current_bet)?;
        let new_ba = new_ba_and_current_bet.0;
        let old_current_bet = self.current_bet;
        self.current_bet = new_ba_and_current_bet.1;
        if self.current_bet > old_current_bet {
            logs.push(LogItem::CurrentBetSet(self.current_bet, self.min_raise));
            // The player has bet more than the current bet.
            //
            // The only reason they can be allowed to not match/exceed the min_raise is if they're
            // allin, and if this is the case, then the min_raise shouldn't be increased. If they
            // met or exceeded the min_raise, it is to be increased.
            if self.current_bet < self.min_raise {
                assert!(matches!(new_ba, BetAction::AllIn(_)));
            } else {
                self.min_raise = self.current_bet + (self.current_bet - old_current_bet);
                logs.push(LogItem::CurrentBetSet(self.current_bet, self.min_raise));
            }
        }
        // Update Pot
        let pot_logs = self.pot.bet(player, new_ba);
        logs.extend(pot_logs.into_iter().map(|pot_logitem| pot_logitem.into()));

        // Determine if this was the final bet and round is over
        if self.seated_players.eligible_players_iter().count() == 1 {
            // If there's only 1 eligible player left, the round isn't just over, but the entire
            // hand is over.
            logs.append(&mut self.finalize_hand()?);
        } else if self.seated_players.is_pot_ready(self.current_bet) {
            // It was the final bet for this round.
            //
            // Advance game state. If 1+ players are NOT all in, then this loop will run once
            // because is_pot_ready(...) will return false. Otherwise all players are all in and
            // this will loop until showdown.
            while self.seated_players.is_pot_ready(self.current_bet)
                && !matches!(self.state, GameState::Showdown)
            {
                let (new_state, mut new_logs) = self.after_bet_advance_round()?;
                self.state = new_state;
                logs.append(&mut new_logs);
                logs.push(LogItem::StateChange(self.state));
            }
            // If that was the end of all betting and we're in showdown, determine winner.
            if matches!(self.state, GameState::Showdown) {
                logs.append(&mut self.finalize_hand()?);
            }
        }
        self.logs_append(logs);
        Ok(())
    }

    /// Simple abstraction so can make big blinds that are not x2 later
    fn big_blind(&self) -> Currency {
        self.small_blind * 2
    }

    fn finalize_hand(&mut self) -> Result<Vec<LogItem>, GameError> {
        let mut logs = vec![];
        let pot = std::mem::take(&mut self.pot);
        // players and their pockets, as a vec
        let players: Vec<(PlayerId, [Card; 2])> = self
            .seated_players
            .eligible_players_iter()
            .map(|sp| (sp.id, sp.pocket.unwrap()))
            .collect();
        // Player ids, sorted in a Vec<Vec<PlayerId>>, for pot's payout function
        let ranked_players = if players.len() == 1 {
            vec![vec![players[0].0]]
        } else {
            assert!(self.table_cards[4].is_some());
            let community = [
                self.table_cards[0].unwrap(),
                self.table_cards[1].unwrap(),
                self.table_cards[2].unwrap(),
                self.table_cards[3].unwrap(),
                self.table_cards[4].unwrap(),
            ];
            let map = players.iter().copied().collect();
            best_hands(&map, community)?
                .iter()
                .map(|inner| inner.iter().map(|item| item.0).collect())
                .collect()
        };
        let (winnings, pot_logs) = pot.payout(&ranked_players);
        self.seated_players.end_hand(&winnings)?;
        self.state = GameState::EndOfHand;
        logs.extend(pot_logs.into_iter().map(|pli| pli.into()));
        logs.push(LogItem::StateChange(self.state));
        // TODO 'stand_up' players who are trying to leave but couldn't because they were in the bet?
        // TODO Force rocket to update DB? Probably by returning State enum?
        Ok(logs)
    }

    /// Move the oldest live [`LogItem`]s to archive logs, and send the oldest archive logs to hell.
    fn rotate_logs(&mut self) {
        assert!(self.max_live_hands >= 1);
        assert!(self.max_archive_hands >= 1);
        let live_hand_starts = hand_starts(&self.live_logs);
        let archive_hand_starts = hand_starts(&self.archive_logs);
        if live_hand_starts.len() as u16 > self.max_live_hands - 1 {
            assert!(live_hand_starts.len() >= 2);
            // split_off splits the vec at the given index, and returns the right-most items.
            // Right-most for us means newest log items, which we really want to keep in live_logs.
            let newest = self.live_logs.split_off(live_hand_starts[1]);
            // So yeah that's why we swap the vecs live_logs and newest here. What was in live_logs
            // is now in oldest.
            let mut oldest = mem::replace(&mut self.live_logs, newest);
            // We got the oldest ones out, now put them in the archvie where they belong!
            self.archive_logs.append(&mut oldest);
        }
        if archive_hand_starts.len() as u16 > self.max_archive_hands - 1 {
            // Same thing as with live hands, but we just drop the oldest archived hands if needed.
            assert!(archive_hand_starts.len() >= 2);
            let newest = self.archive_logs.split_off(archive_hand_starts[1]);
            self.archive_logs = newest;
        }
    }

    /// Returns an iterator that returns live [`LogItem`]s that have sequence numbers greater than
    /// the given one.
    ///
    /// This function returns **live** logs, not archive logs.
    ///
    /// The very first log item ever recorded in a GameInProgress will have sequence number 1.
    /// Thus in all cases, passing 0 to this function will return all live logs. If the sequence
    /// number provided is greated than that of all live logs, the reteurned iterator is empty.
    pub fn logs_since(&mut self, seq_num: usize) -> impl Iterator<Item = (usize, LogItem)> + '_ {
        self.live_logs
            .iter()
            .skip_while(move |item| item.0 < seq_num)
            .cloned()
    }

    /// Determine our most recent log item's sequence number and return one more than that, else 1.
    fn next_log_seq_num(&self) -> usize {
        let live_logs_len = self.live_logs.len();
        let archive_logs_len = self.archive_logs.len();
        if live_logs_len > 0 {
            self.live_logs[live_logs_len - 1].0 + 1
        } else if archive_logs_len > 0 {
            self.archive_logs[archive_logs_len - 1].0 + 1
        } else {
            1
        }
    }

    fn logs_push(&mut self, log: LogItem) {
        self.live_logs.push((self.next_log_seq_num(), log));
    }

    fn logs_append(&mut self, logs: Vec<LogItem>) {
        for log in logs {
            self.live_logs.push((self.next_log_seq_num(), log));
        }
    }

    #[cfg(test)]
    fn live_logs(&self) -> &Vec<(usize, LogItem)> {
        &self.live_logs
    }

    #[cfg(test)]
    fn archive_logs(&self) -> &Vec<(usize, LogItem)> {
        &self.archive_logs
    }
}

/// Helper function for [`GameInProgress::rotate_logs`] that pulls out only the events that signify
/// the start of a hand.
fn hand_starts(logs: &[(usize, LogItem)]) -> Vec<usize> {
    logs.iter()
        .enumerate()
        .filter_map(|(idx, (_, li))| {
            if matches!(li, LogItem::StateChange(GameState::Dealing)) {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::DeckSeed;

    fn seed1() -> DeckSeed {
        DeckSeed::new([1; 32])
    }

    fn basic_table() -> GameInProgress {
        let mut gt = GameInProgress::default();
        gt.sit_down(0, 1000, 0).unwrap(); // dealer
        gt.sit_down(1, 1000, 1).unwrap(); // small blind
        gt.sit_down(2, 1000, 2).unwrap(); // big blind
        gt.sit_down(3, 1000, 3).unwrap();
        gt.start_round(&seed1()).unwrap();
        gt
    }

    #[test]
    fn minraise() {
        let mut gt = basic_table();
        gt.bet(3, BetAction::Raise(30.into())).unwrap();
        assert_eq!(gt.min_raise, 50.into());
        gt.bet(0, BetAction::Raise(60.into())).unwrap();
        assert_eq!(gt.min_raise, 90.into());
        gt.bet(1, BetAction::Raise(200.into())).unwrap();
        assert_eq!(gt.min_raise, 340.into());
    }

    /// A player going all in for less than the minimum raise does not change the minimum raise.
    /// Furthermore, the original raiser does not get the chance to raise again after them
    /// (because they'd be raising themselves).
    #[test]
    fn minraise_fullbet_rule() {
        let mut gt = GameInProgress::default();
        gt.sit_down(0, 600, 0).unwrap(); // dealer
        gt.sit_down(1, 1000, 1).unwrap(); // small blind
        gt.sit_down(2, 1000, 2).unwrap(); // big blind
        gt.sit_down(3, 1000, 3).unwrap();
        gt.start_round(&seed1()).unwrap();
        // First raise. Not the critical moment
        gt.bet(3, BetAction::Raise(500.into())).unwrap();
        assert_eq!(gt.min_raise, 990.into());
        assert_eq!(gt.current_bet, 500.into());
        // Second "raise" that's an all in. This is the first critical moment. The min raise
        // shouldn't change, but current_bet should.
        gt.bet(0, BetAction::AllIn(600.into())).unwrap();
        assert_eq!(gt.min_raise, 990.into());
        assert_eq!(gt.current_bet, 600.into());

        // player 1 gets out of the way and nothing changes
        gt.bet(1, BetAction::Fold).unwrap();
        assert_eq!(gt.min_raise, 990.into());
        assert_eq!(gt.current_bet, 600.into());
        // player 2 calls and again nothing changes
        gt.bet(2, BetAction::Call(600.into())).unwrap();
        assert_eq!(gt.min_raise, 990.into());
        assert_eq!(gt.current_bet, 600.into());

        // player 3 can't raise because that'd be raising themself. This is the second critial
        // moment.
        let e = gt.bet(3, BetAction::Raise(990.into())).unwrap_err();
        assert!(matches!(e, GameError::BetError(BetError::CantRaiseSelf)));
    }

    #[test]
    fn basic_game() {
        let mut gt = GameInProgress::default();
        gt.sit_down(0, 100, 0).unwrap(); // dealer
        gt.sit_down(1, 100, 1).unwrap(); // small blind
        gt.sit_down(2, 100, 2).unwrap(); // big blind
        gt.sit_down(3, 100, 3).unwrap();
        gt.start_round(&seed1()).unwrap();
        // Blinds are in
        assert!(matches!(gt.state, GameState::Betting(BetRound::PreFlop)));
        assert_eq!(gt.get_player_info(0).unwrap().monies, 100.into());
        assert_eq!(gt.get_player_info(1).unwrap().monies, 95.into());
        assert_eq!(gt.get_player_info(2).unwrap().monies, 90.into());
        assert_eq!(gt.pot.total_value(), 15.into());
        assert_eq!(gt.seated_players.dealer_token, 0);
        assert_eq!(gt.seated_players.small_blind_token, 1);
        assert_eq!(gt.seated_players.big_blind_token, 2);

        gt.bet(3, BetAction::Call(10.into())).unwrap();
        gt.bet(0, BetAction::Fold).unwrap();
        gt.bet(1, BetAction::Call(10.into())).unwrap();
        gt.bet(2, BetAction::Check).unwrap();

        // First betting round is over.
        // Table should recognize that all players are in and pot is right and forward the round
        assert_eq!(gt.get_player_info(0).unwrap().monies, 100.into());
        assert_eq!(gt.get_player_info(1).unwrap().monies, 90.into());
        assert_eq!(gt.get_player_info(2).unwrap().monies, 90.into());
        assert_eq!(gt.get_player_info(3).unwrap().monies, 90.into());
        assert_eq!(gt.pot.total_value(), 30.into());
        assert!(gt.table_cards[0].is_some());
        assert!(gt.table_cards[1].is_some());
        assert!(gt.table_cards[2].is_some());
        assert!(gt.table_cards[3].is_none());
        assert!(gt.table_cards[4].is_none());
        assert!(matches!(gt.state, GameState::Betting(BetRound::Flop)));

        gt.bet(1, BetAction::Check).unwrap();
        gt.bet(2, BetAction::Bet(20.into())).unwrap();
        gt.bet(3, BetAction::Call(20.into())).unwrap();
        // 0 folded
        gt.bet(1, BetAction::Call(20.into())).unwrap();

        // Second betting round is over.
        assert_eq!(gt.get_player_info(0).unwrap().monies, 100.into());
        assert_eq!(gt.get_player_info(1).unwrap().monies, 70.into());
        assert_eq!(gt.get_player_info(2).unwrap().monies, 70.into());
        assert_eq!(gt.get_player_info(3).unwrap().monies, 70.into());
        assert_eq!(gt.pot.total_value(), (30 + 60).into());
        assert!(gt.table_cards[0].is_some());
        assert!(gt.table_cards[1].is_some());
        assert!(gt.table_cards[2].is_some());
        assert!(gt.table_cards[3].is_some());
        assert!(gt.table_cards[4].is_none());
        assert!(matches!(gt.state, GameState::Betting(BetRound::Turn)));

        gt.bet(1, BetAction::Bet(30.into())).unwrap();
        gt.bet(2, BetAction::Call(30.into())).unwrap();
        gt.bet(3, BetAction::Raise(60.into())).unwrap();
        gt.bet(1, BetAction::Call(60.into())).unwrap();
        gt.bet(2, BetAction::Fold).unwrap();

        // Third betting round is over.
        assert_eq!(gt.get_player_info(0).unwrap().monies, 100.into());
        assert_eq!(gt.get_player_info(1).unwrap().monies, 10.into());
        assert_eq!(gt.get_player_info(2).unwrap().monies, 40.into());
        assert_eq!(gt.get_player_info(3).unwrap().monies, 10.into());
        assert_eq!(gt.pot.total_value(), (90 + 150).into());
        assert!(gt.table_cards[0].is_some());
        assert!(gt.table_cards[1].is_some());
        assert!(gt.table_cards[2].is_some());
        assert!(gt.table_cards[3].is_some());
        assert!(gt.table_cards[4].is_some());
        assert!(matches!(gt.state, GameState::Betting(BetRound::River)));

        gt.bet(1, BetAction::AllIn(10.into())).unwrap();
        gt.bet(3, BetAction::Call(10.into())).unwrap(); // AllIn should also work

        // This should be the end of the hand. Winner should be paid out. Etc.
        // We can rely on these payouts because we have the same deck seed every time.
        dbg!(&gt);
        assert_eq!(gt.get_player_info(0).unwrap().monies, 100.into());
        assert_eq!(gt.get_player_info(1).unwrap().monies, 0.into());
        assert_eq!(gt.get_player_info(2).unwrap().monies, 40.into());
        assert_eq!(gt.get_player_info(3).unwrap().monies, 260.into());

        for (n, item) in gt.live_logs().into_iter() {
            println!("{:2}: {}", n, item);
        }
    }

    // TODO test where players who are folded try to bet again

    // TODO test where players that are currently bet eligible try to stand up

    // TODO moar
}

#[cfg(test)]
mod test_logs {
    use super::*;

    fn play_one_hand(gt: &mut GameInProgress) {
        gt.start_round(&DeckSeed::default()).unwrap();
        let player = gt.next_player().unwrap();
        gt.bet(player, BetAction::Fold).unwrap();
    }

    #[test]
    fn live_logs_len() {
        let mut gt = GameInProgress::default();
        gt.sit_down(0, 10000, 0).unwrap();
        gt.sit_down(1, 10000, 1).unwrap();
        assert_eq!(hand_starts(gt.live_logs()).len(), 0);
        // hands accumulate in live_logs
        for n in 0..=DEF_MAX_LIVE_HANDS {
            assert_eq!(hand_starts(gt.live_logs()).len(), n as usize);
            play_one_hand(&mut gt);
        }
        assert_eq!(
            hand_starts(gt.live_logs()).len(),
            DEF_MAX_LIVE_HANDS as usize
        );
        // as more hands are played, live_logs still doesn't get bigger than one more than
        // DEF_MAX_LIVE_HANDS
        play_one_hand(&mut gt);
        assert_eq!(
            hand_starts(gt.live_logs()).len(),
            DEF_MAX_LIVE_HANDS as usize
        );
        play_one_hand(&mut gt);
        assert_eq!(
            hand_starts(gt.live_logs()).len(),
            DEF_MAX_LIVE_HANDS as usize
        );
        play_one_hand(&mut gt);
        play_one_hand(&mut gt);
        assert_eq!(
            hand_starts(gt.live_logs()).len(),
            DEF_MAX_LIVE_HANDS as usize
        );
    }

    #[test]
    fn archive_logs_len() {
        let mut gt = GameInProgress::default();
        gt.sit_down(0, 10000, 0).unwrap();
        gt.sit_down(1, 10000, 1).unwrap();
        assert_eq!(hand_starts(gt.archive_logs()).len(), 0);
        // first few hands don't touch archive_logs right away
        for _ in 0..DEF_MAX_LIVE_HANDS {
            assert_eq!(hand_starts(gt.archive_logs()).len(), 0);
            play_one_hand(&mut gt);
        }
        // still haven't rotated
        assert_eq!(hand_starts(gt.archive_logs()).len(), 0);
        // but now they start, since live_logs is starting to overflow into archive
        for n in 0..DEF_MAX_ARCHIVE_HANDS {
            assert_eq!(hand_starts(gt.archive_logs()).len(), n as usize);
            play_one_hand(&mut gt);
        }
        // archive is full, and additional games don't overfill it
        assert_eq!(
            hand_starts(gt.archive_logs()).len(),
            DEF_MAX_ARCHIVE_HANDS as usize
        );
        play_one_hand(&mut gt);
        assert_eq!(
            hand_starts(gt.archive_logs()).len(),
            DEF_MAX_ARCHIVE_HANDS as usize
        );
        play_one_hand(&mut gt);
        assert_eq!(
            hand_starts(gt.archive_logs()).len(),
            DEF_MAX_ARCHIVE_HANDS as usize
        );
        play_one_hand(&mut gt);
        play_one_hand(&mut gt);
        assert_eq!(
            hand_starts(gt.archive_logs()).len(),
            DEF_MAX_ARCHIVE_HANDS as usize
        );

        for (n, log) in gt.logs_since(0) {
            println!("{n} {log}");
        }
        //assert!(false);
    }

    #[test]
    fn logs_start_at_one() {
        let mut gt = GameInProgress::default();
        gt.sit_down(0, 10000, 0).unwrap();
        gt.sit_down(1, 10000, 1).unwrap();
        play_one_hand(&mut gt);
        let first = gt.logs_since(0).nth(0).unwrap().0;
        assert_eq!(first, 1);
    }

    #[test]
    fn logs_are_sequential() {
        let mut gt = GameInProgress::default();
        gt.sit_down(0, 10000, 0).unwrap();
        gt.sit_down(1, 10000, 1).unwrap();
        play_one_hand(&mut gt);
        let mut iter = gt.logs_since(0).map(|(i, _)| i);
        let mut prev = iter.nth(0).unwrap();
        for n in iter {
            assert_eq!(n, prev + 1);
            prev = n;
        }
    }

    #[test]
    fn logs_since_through_rotate() {
        // read all live logs, play another hand s.t. live_logs has to overflow into archive, and
        // try reading new live logs. There should be some returned.
        let mut gt = GameInProgress::default();
        gt.sit_down(0, 10000, 0).unwrap();
        gt.sit_down(1, 10000, 1).unwrap();
        for _ in 0..DEF_MAX_LIVE_HANDS {
            play_one_hand(&mut gt);
        }
        // get seq num of last log
        let last = gt.logs_since(0).last().unwrap().0;
        // play another hand and make sure we get some logs back.
        play_one_hand(&mut gt);
        assert!(gt.logs_since(last).count() > 0);
    }

    #[test]
    fn logs_since_too_big() {
        let mut gt = GameInProgress::default();
        gt.sit_down(0, 10000, 0).unwrap();
        gt.sit_down(1, 10000, 1).unwrap();
        play_one_hand(&mut gt);
        assert_eq!(gt.logs_since(1_000_000).count(), 0);
    }
}
