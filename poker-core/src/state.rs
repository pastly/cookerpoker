use crate::bet::BetAction;
use crate::cards::{best_hands, Card, Deck, DeckSeed, Hand};
use crate::log::{Log, LogItem};
use crate::player::{Player, PlayerFilter, Players};
use crate::pot::Pot;
use crate::{Currency, GameError, PlayerId, SeatIdx, SeqNum, MAX_PLAYERS};
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

const COMMUNITY_SIZE: usize = 5;
const DEF_SB: Currency = 5;
const DEF_BB: Currency = 10;

type PidBA = (PlayerId, BetAction);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum TableType {
    Cash,
}

impl Default for TableType {
    fn default() -> Self {
        Self::Cash
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseState {
    pub table_type: TableType,
    pub seats: [Option<Player>; MAX_PLAYERS],
}

impl std::fmt::Display for BaseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {}",
            self.table_type,
            self.seats.iter().filter(|p| p.is_some()).count()
        )
    }
}

impl From<&mut GameState> for BaseState {
    fn from(gs: &mut GameState) -> Self {
        let mut seats = [None; MAX_PLAYERS];
        let seats = {
            for (idx, p) in gs.players.players_iter(PlayerFilter::ALL) {
                seats[idx] = Some(*p);
            }
            seats
        };
        Self {
            table_type: gs.table_type,
            seats,
        }
    }
}

/// States a game can be in, e.g. not even stardard, dealing, showdown, etc.
#[derive(Debug, PartialEq, Eq, Clone, Copy, derive_more::Display, Serialize, Deserialize)]
pub enum State {
    NotStarted,
    Dealing,
    Street(Street),
    Showdown,
    EndOfHand,
}

impl Default for State {
    fn default() -> Self {
        Self::NotStarted
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, derive_more::Display, Serialize, Deserialize)]
pub enum Street {
    PreFlop,
    Flop,
    Turn,
    River,
}

/// (Replaces GameInProgress) All the state constituting a poker game in progress
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    /// The state this Game is in ... as in what street or showdown or paused
    __state_dont_change_directly: State,
    /// Cash. Maybe tourny in the future
    pub table_type: TableType,
    /// The players seated at this table and their per-player info
    pub players: Players,
    /// The community cards
    pub community: [Option<Card>; COMMUNITY_SIZE],
    /// Management of the pot and any side pots
    pot: Pot,
    /// The deck, obviously.
    deck: Deck,
    /// The small blind, obviously.
    small_blind: Currency,
    /// The big blind, obviously.
    big_blind: Currency,
    /// The amount that each player is expected to match in order to make it to the end of the
    /// current betting round.
    __current_bet_dont_change_directly: Currency,
    /// If a player wishes to raise this betting round, they must raise to at least this amount.
    /// This is the total amount to raise to, i.e. it is larger than current_bet.
    __min_raise_dont_change_directly: Currency,
    /// The last person to raise this betting round.
    ///
    /// Needed because of the full bet rule. You can't raise, have action come back to you, then
    /// raise again without someone raising after your first raise. Action can come back to you
    /// like this if someone goes all in for less than the minimum raise after your first raise.
    ///
    /// It's confusing. See <https://duckduckgo.com/?t=ffab&q=allin+raise+less+than+minraise>
    last_raiser: Option<PlayerId>,
    /// Logs since the the start of this hand and an archive of some previous hands
    logs: Log,
}

impl GameState {
    pub fn filtered_changes_since(
        &self,
        seq: SeqNum,
        player_id: PlayerId,
    ) -> impl Iterator<Item = (SeqNum, LogItem)> + '_ {
        self.logs
            .items_since(seq)
            .map(move |(idx, item)| match item {
                LogItem::Pot(_)
                | LogItem::NewBaseState(_)
                | LogItem::StateChange(_, _)
                | LogItem::TokensSet(_, _, _)
                | LogItem::NextToAct(_)
                | LogItem::CurrentBetSet(_, _, _, _)
                | LogItem::HandReveal(_, _)
                | LogItem::Flop(_, _, _)
                | LogItem::Turn(_)
                | LogItem::River(_) => (idx, item),
                LogItem::PocketDealt(pid, _pocket) => {
                    if pid == player_id {
                        (idx, item)
                    } else {
                        (idx, LogItem::PocketDealt(pid, None))
                    }
                }
            })
    }

    //#[cfg(test)]
    //pub(crate) fn changes_since(
    //    &self,
    //    seq: SeqNum,
    //) -> impl Iterator<Item = (SeqNum, LogItem)> + '_ {
    //    self.logs.items_since(seq)
    //}

    pub fn pot_total_value(&self) -> Currency {
        self.pot.total_value()
    }

    pub fn nta(&self) -> Option<(SeatIdx, Player)> {
        match self.players.need_bets_from.is_empty() {
            false => {
                let idx = self.players.need_bets_from[self.players.need_bets_from.len() - 1];
                let p = self.players.players[idx].unwrap();
                Some((idx, p))
            }
            true => None,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            __state_dont_change_directly: Default::default(),
            table_type: Default::default(),
            players: Default::default(),
            community: [None; COMMUNITY_SIZE],
            pot: Default::default(),
            deck: Default::default(),
            small_blind: DEF_SB,
            big_blind: DEF_BB,
            __current_bet_dont_change_directly: DEF_BB,
            __min_raise_dont_change_directly: 2 * DEF_BB,
            last_raiser: None,
            logs: Default::default(),
        }
    }
}

impl GameState {
    pub fn player_folds(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        self.player_action(player_id, BetAction::Fold)
    }

    pub fn player_calls(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        self.player_action(player_id, BetAction::Call(self.current_bet()))
    }

    pub fn player_checks(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        self.player_action(player_id, BetAction::Check)
    }

    pub fn player_bets(&mut self, player_id: PlayerId, val: Currency) -> Result<(), GameError> {
        self.player_action(player_id, BetAction::Bet(val))
    }

    pub fn player_raises(&mut self, player_id: PlayerId, val: Currency) -> Result<(), GameError> {
        self.player_action(player_id, BetAction::Raise(val))
    }

    pub fn player_action(
        &mut self,
        player_id: PlayerId,
        bet_action: BetAction,
    ) -> Result<(), GameError> {
        let bet = self.bet(player_id, bet_action)?;
        // based on the bet's value, update current_bet and min_raise if needed
        let bet_value = match bet {
            BetAction::Check | BetAction::Fold => 0,
            BetAction::Call(v) | BetAction::Bet(v) | BetAction::Raise(v) | BetAction::AllIn(v) => v,
        };
        if bet_value > self.current_bet() {
            let old_cb = self.current_bet();
            let cb = bet_value;
            let mr = cb + (cb - old_cb);
            self.set_current_bet(cb, mr);
        }
        let mut pot_logs = vec![];
        pot_logs.append(&mut self.pot.bet(player_id, bet));
        self.logs.extend(pot_logs.into_iter().map(|l| l.into()));

        if self
            .players
            .players_iter(PlayerFilter::POT_ELIGIBLE)
            .count()
            == 1
        {
            self.finalize_hand()?;
        } else if self.players.need_bets_from.is_empty() {
            while self.players.need_bets_from.is_empty() && !matches!(self.state(), State::Showdown)
            {
                let next_state = self.advance_street()?;
                self.change_state(next_state);
            }
            if matches!(self.state(), State::Showdown) {
                self.finalize_hand()?;
            }
        }
        if !self.players.need_bets_from.is_empty() {
            self.logs.push(LogItem::NextToAct(self.nta().unwrap().0));
        }
        Ok(())
    }

    fn change_state(&mut self, new: State) {
        self.logs
            .push(LogItem::StateChange(self.__state_dont_change_directly, new));
        // this is the only place the state should ever be changed directly
        self.__state_dont_change_directly = new;
    }

    fn set_current_bet(&mut self, new_cb: Currency, new_mr: Currency) {
        let old_cb = self.__current_bet_dont_change_directly;
        let old_mr = self.__min_raise_dont_change_directly;
        self.logs
            .push(LogItem::CurrentBetSet(old_cb, new_cb, old_mr, new_mr));
        // this is the only place these should ever be changed directly
        self.__current_bet_dont_change_directly = new_cb;
        self.__min_raise_dont_change_directly = new_mr;
    }

    pub const fn state(&self) -> State {
        self.__state_dont_change_directly
    }

    pub const fn current_bet(&self) -> Currency {
        self.__current_bet_dont_change_directly
    }

    pub const fn min_raise(&self) -> Currency {
        self.__min_raise_dont_change_directly
    }

    fn advance_street(&mut self) -> Result<State, GameError> {
        let next = match self.state() {
            State::Street(round) => match round {
                Street::PreFlop => State::Street(Street::Flop),
                Street::Flop => State::Street(Street::Turn),
                Street::Turn => State::Street(Street::River),
                Street::River => State::Showdown,
            },
            _ => unreachable!(),
        };
        self.players.next_street()?;
        let pot_logs = self.pot.finalize_round();
        self.logs.extend(pot_logs.into_iter().map(|l| l.into()));
        self.set_current_bet(0, self.big_blind);
        self.last_raiser = None;
        if let State::Street(street) = next {
            match street {
                Street::PreFlop => unreachable!(),
                Street::Flop => {
                    self.deck.burn();
                    let c1 = self.deck.draw();
                    let c2 = self.deck.draw();
                    let c3 = self.deck.draw();
                    self.community[0] = Some(c1);
                    self.community[1] = Some(c2);
                    self.community[2] = Some(c3);
                    self.logs.push(LogItem::Flop(c1, c2, c3));
                }
                Street::Turn => {
                    self.deck.burn();
                    let c1 = self.deck.draw();
                    self.community[3] = Some(c1);
                    self.logs.push(LogItem::Turn(c1));
                }
                Street::River => {
                    self.deck.burn();
                    let c1 = self.deck.draw();
                    self.community[4] = Some(c1);
                    self.logs.push(LogItem::River(c1));
                }
            }
        }
        Ok(next)
    }

    pub fn try_sit(&mut self, player_id: PlayerId, stack: Currency) -> Result<(), GameError> {
        if self.players.player_by_id(player_id).is_some() {
            return Err(GameError::PlayerAlreadySeated);
        }
        let p = Player::new(player_id, stack);
        self.players.seat_player(p)?;
        Ok(())
    }

    /// If we are able to automatically move the current game forward, do so
    pub fn tick(&mut self) -> Result<(), GameError> {
        // If there's no game going and there's enough people to start one, do so
        if matches!(self.state(), State::NotStarted)
            && self.players.players_iter(PlayerFilter::MAY_BET).count() > 1
        {
            return self.start_hand();
        }
        // If it's the end of a hand, start a new one
        if matches!(self.state(), State::EndOfHand) {
            return self.start_hand();
        }
        Ok(())
    }

    fn finalize_hand(&mut self) -> Result<(), GameError> {
        let pot = std::mem::take(&mut self.pot);
        // players and their pockets, as a vec
        let players: Vec<(PlayerId, Hand)> = self
            .players
            .players_iter(PlayerFilter::POT_ELIGIBLE)
            .map(|(_, p)| (p.id, p.hand.expect("Tried to finalize empty hand")))
            .collect();
        // PlayerIds, sorted in a Vec<Vec<PlayerId>>, for pot's payout function
        let ranked_players = if players.len() == 1 {
            vec![vec![players[0].0]]
        } else {
            assert!(self.community[4].is_some());
            let map = players.iter().copied().collect();
            best_hands(&map)
                .iter()
                .map(|inner| inner.iter().map(|item| item.0).collect())
                .collect()
        };
        let (winnings, pot_logs) = pot.payout(&ranked_players);
        // determine who needs to reveal their hand to win, if anybody, and log the reveal. A hand
        // needs to be revealed if there's more than 1 person that could win the pot at this time.
        if players.len() > 1 {
            for winning_player_id in winnings.keys() {
                let p = self
                    .players
                    .player_by_id(*winning_player_id)
                    .expect("Unable to get player that allegedly won (at least part of) the pot");
                let cards = p
                    .hand
                    .expect("player that won (at least part of) the pot has no cards")
                    .pocket
                    .expect("player that won (at least part of) the pot has no cards");
                let li = LogItem::HandReveal(*winning_player_id, [Some(cards[0]), Some(cards[1])]);
                self.logs.push(li);
            }
        }
        self.players.end_hand(&winnings)?;
        self.change_state(State::EndOfHand);
        self.logs.extend(pot_logs.into_iter().map(|pli| pli.into()));
        Ok(())
    }

    fn clean_state(&mut self, deck_seed: DeckSeed) {
        self.logs.rotate();
        self.players.clean_state();
        let bs = Box::new(self.into());
        self.logs.push(LogItem::NewBaseState(bs));
        self.change_state(State::NotStarted);
        self.community = [None; COMMUNITY_SIZE];
        self.pot = Default::default();
        self.deck = Deck::new(deck_seed);
        self.set_current_bet(0, self.big_blind);
        self.last_raiser = None;
    }

    pub fn start_hand(&mut self) -> Result<(), GameError> {
        let seed = DeckSeed::default();
        self.start_hand_with_seed(seed)
    }

    pub fn start_hand_with_seed(&mut self, seed: DeckSeed) -> Result<(), GameError> {
        self.clean_state(seed);
        self.players.start_hand()?;
        self.change_state(State::Street(Street::PreFlop));
        self.logs.push(LogItem::TokensSet(
            self.players.token_dealer,
            self.players.token_sb,
            self.players.token_bb,
        ));
        self.set_current_bet(0, self.big_blind);
        let ((player_sb, bet_sb), (player_bb, bet_bb)) = self.blinds_bet()?;
        let mut pot_logs = vec![];
        pot_logs.append(&mut self.pot.bet(player_sb, bet_sb));
        pot_logs.append(&mut self.pot.bet(player_bb, bet_bb));
        self.logs.extend(pot_logs.into_iter().map(|l| l.into()));
        self.set_current_bet(self.big_blind, self.big_blind * 2);
        // at this point, there is no last raiser, but the bet function thinks there is (it considers
        // the BB to have taken the most recent agressive action). Thus we won't let the BB raise if
        // no one raises before him ... unless we clear the last_raiser.
        // We assert here because if logic changes, we might be able to clean this up, or we might
        // be fucking something up.
        assert!(self.last_raiser.is_some());
        assert_eq!(
            self.last_raiser.unwrap(),
            self.players.players[self.players.token_bb].unwrap().id,
        );
        self.last_raiser = None;

        let num_p = self.players.players_iter(PlayerFilter::MAY_BET).count() as u8;
        let pockets = self.deck.deal_pockets(num_p);
        // TODO don't know how I feel about logging the pocket values
        /*let deal_logs = self
            .players
            .deal_pockets(pockets)
            .into_iter()
            .map(|(k, v)| LogItem::PocketDealt(k, v));
        self.logs.extend(deal_logs);
        */
        self.logs.push(LogItem::NextToAct(self.nta().unwrap().0));
        Ok(())
    }

    /// Have the SB and BB execute their obligatory preflop betting. Return their IDs and bet
    /// amounts.
    ///
    /// Caller can't assume SB and BB are in for the full SB/BB amount: they could have been a very
    /// short stack and now be allin for less.
    fn blinds_bet(&mut self) -> Result<(PidBA, PidBA), GameError> {
        let player_sb =
            self.players.players[self.players.token_sb].ok_or(GameError::PlayerNotFound)?;
        let player_bb =
            self.players.players[self.players.token_bb].ok_or(GameError::PlayerNotFound)?;
        let bet_sb = self.bet(player_sb.id, BetAction::Bet(self.small_blind))?;
        let bet_bb = self.bet(player_bb.id, BetAction::Bet(self.big_blind))?;
        // the blinds have bet, and we need to make sure they have the opportunity to bet again this
        // round, so rebuild need_bets_from
        self.players.need_bets_from = self
            .players
            .betting_players_iter_after(self.players.token_bb)
            .map(|(i, _)| i)
            .take(self.players.players_iter(PlayerFilter::MAY_BET).count())
            .collect();
        self.players.need_bets_from.reverse();
        Ok(((player_sb.id, bet_sb), (player_bb.id, bet_bb)))
    }

    /// Check that the player can make the given bet, adjusting it if possible. Returns the
    /// (possibly adjusted) bet this player made
    fn bet(&mut self, player_id: PlayerId, bet: BetAction) -> Result<BetAction, GameError> {
        // Check for obvious errors: game not in correct state
        if !matches!(self.state(), State::Street(_)) {
            return Err(GameError::NoBetExpected);
        }
        // Check for obvious errors: bet too small, or this player shouldn't be betting, etc.
        match &bet {
            // nothing obvious to check for
            BetAction::Check | BetAction::Fold => {}
            // can be for any amount, so no errors to catch
            BetAction::AllIn(_) => {}
            BetAction::Bet(x) | BetAction::Call(x) => {
                match x.cmp(&self.current_bet()) {
                    Ordering::Less => return Err(GameError::InvalidBet),
                    Ordering::Greater => {
                        // only an error if there is a non-zero current bet. It's 0 for the start of
                        // post-flop rounds
                        if self.current_bet() != 0 {
                            return Err(GameError::InvalidBet);
                        }
                    }
                    // No errors to account for and no maintenance to do
                    Ordering::Equal => {}
                }
            }
            BetAction::Raise(x) => {
                if x < &self.min_raise() {
                    return Err(GameError::InvalidBet);
                }
                // Cannot raise if same player was most recent player to raise
                if self.last_raiser.is_some() && self.last_raiser.unwrap() == player_id {
                    return Err(GameError::InvalidBet);
                }
            }
        }
        // More error checks bundled with grabbing the seat index of this player. Stupidness here
        // because we don't want to maintain a borrow
        let seat = {
            let (seat, p) = self
                .players
                .player_with_index_by_id(player_id)
                .ok_or(GameError::PlayerNotFound)?;
            if !p.is_betting() {
                return Err(GameError::PlayerIsNotBetting);
            } else if self.players.need_bets_from.is_empty() {
                // perhaps the round should have been marked as ended?
                return Err(GameError::NoBetExpected);
            } else if self.players.need_bets_from[self.players.need_bets_from.len() - 1] != seat {
                // the next player we expect a bet from is the last item in the list
                return Err(GameError::OutOfTurn);
            }
            seat
        };
        // Determine if we should update the last_raiser, assuming we get through the rest of this
        // function without error
        let should_update_last_raiser = match &bet {
            BetAction::Check | BetAction::Fold => false,
            BetAction::Call(x) | BetAction::Bet(x) | BetAction::Raise(x) | BetAction::AllIn(x) => {
                // it should be safe and correct to check all these bet types, even if we only
                // expect allin/raise
                *x >= self.min_raise()
            }
        };

        // There are no more obvious issues. Assuming the player has enough in their stack, have
        // them take the bet from their stack (updates their stack size) and convert the bet to an
        // allin if needed.
        let bet = self
            .players
            .player_by_id_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?
            .bet(bet)?;

        // If the bet is for an amount greater than the current bet, then a full orbit is required
        // to give everyone a chance to match it. We expect it to be ...
        // - equal for calls,
        // - less for allin-for-less,
        // - more for raises (incl allin raises)
        match bet {
            BetAction::Check | BetAction::Fold => {
                self.players.need_bets_from.pop();
            }
            BetAction::Call(x) | BetAction::Bet(x) | BetAction::Raise(x) | BetAction::AllIn(x) => {
                match x.cmp(&self.current_bet()) {
                    std::cmp::Ordering::Less => {
                        // the only time less is ok is if this is allin
                        if bet.is_allin() {
                            self.players.need_bets_from.pop();
                        } else {
                            return Err(GameError::InvalidBet);
                        }
                    }
                    std::cmp::Ordering::Equal => {
                        self.players.need_bets_from.pop();
                    }
                    std::cmp::Ordering::Greater => {
                        // if this player just went all in, then there's one less betting player
                        // left than if this was a raise (b/c they can't do any more actions if
                        // they're allin)
                        let n = if bet.is_allin()
                            && self.players.players_iter(PlayerFilter::MAY_BET).count() == 0
                        {
                            0
                        } else if bet.is_allin() {
                            self.players.players_iter(PlayerFilter::MAY_BET).count()
                        } else {
                            self.players.players_iter(PlayerFilter::MAY_BET).count() - 1
                        };
                        self.players.need_bets_from = self
                            .players
                            .betting_players_iter_after(seat)
                            .map(|(i, _)| i)
                            .take(n)
                            .collect();
                        self.players.need_bets_from.reverse();
                    }
                }
            }
        }

        if should_update_last_raiser {
            self.last_raiser = Some(player_id);
        }
        Ok(bet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bet::BetStatus;
    use crate::player::Player;
    use crate::MAX_PLAYERS;

    #[test]
    fn all_in_on_blind() {
        let mut gs = GameState::default();
        gs.players.players[0] = Some(Player::new(1, 2));
        gs.players.players[5] = Some(Player::new(2, 10));
        gs.start_hand().unwrap();
        assert_eq!(
            gs.players.player_by_id(1).unwrap().bet_status,
            BetStatus::AllIn(2)
        );
        assert_eq!(
            gs.players.player_by_id(2).unwrap().bet_status,
            BetStatus::In(DEF_SB)
        );
    }

    #[test]
    fn player_cant_sit_twice() {
        let mut gs = GameState::default();
        gs.try_sit(1, 10).unwrap();
        let r = gs.try_sit(1, 123);
        assert!(r.is_err());
    }

    /// deal_pockets function doesn't panic, likely because it's trying to deal more pockets than
    /// it was given (by giving the same person two pockets)
    #[test]
    fn deal_pockets() {
        // make sure it works for a variety of number of players
        for n_players in 2..=MAX_PLAYERS {
            // make sure it works when any player is the first one
            for first in 0..n_players {
                let mut gs = GameState::default();
                for seat in 0..n_players {
                    gs.try_sit(seat as PlayerId, 10000).unwrap();
                }
                // move dealer token to correct player
                while gs.players.token_dealer != first as SeatIdx {
                    gs.players.start_hand().unwrap();
                }
                let mut deck = Deck::default();
                let pockets = deck.deal_pockets(n_players as u8);
                // this is the actual test. Does this panic?
                gs.players.deal_pockets(pockets);
                // okay so it didn't. let's make sure every player has a pocket.
                for (_, player) in gs.players.players_iter(PlayerFilter::ALL) {
                    assert!(player.hand.is_some());
                }
            }
        }
    }

    /// When action folds to the SB and the SB just completes, the BB is allowed to raise
    #[test]
    fn bigblind_can_raise() {
        let mut gs = GameState::default();
        const STACK: Currency = DEF_BB * 10;
        const SB_PID: PlayerId = 1;
        const BB_PID: PlayerId = 2;
        gs.try_sit(BB_PID, STACK).unwrap();
        gs.try_sit(SB_PID, STACK).unwrap();
        gs.start_hand().unwrap();
        const SB_SEAT: SeatIdx = 1;
        const BB_SEAT: SeatIdx = 0;
        // sanity checks
        assert_eq!(gs.players.token_dealer, SB_SEAT);
        assert_eq!(gs.players.token_sb, SB_SEAT);
        assert_eq!(gs.players.token_bb, BB_SEAT);
        assert_eq!(gs.nta().unwrap().0, SB_SEAT);
        // sb completes, action now on bb
        gs.player_calls(SB_PID).unwrap();
        // sanity check: bb is nta
        assert_eq!(gs.nta().unwrap().0, BB_SEAT);
        // the test: bb is allowed to raise
        gs.player_raises(BB_PID, DEF_BB * 3).unwrap();
    }
}
