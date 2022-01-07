use super::{deck::Card, BetAction, BetError, Currency, GameError};
use std::cmp::Ordering;
use std::collections::HashMap;
pub const MAX_PLAYERS: usize = 12;

pub type PlayerId = i32;

type PlayerBetAction = (PlayerId, BetAction);

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BetStatus {
    Folded,
    Waiting,
    In(Currency),
    AllIn(Currency),
}

impl From<BetAction> for BetStatus {
    fn from(ba: BetAction) -> Self {
        match ba {
            BetAction::AllIn(x) => BetStatus::AllIn(x),
            BetAction::Fold => BetStatus::Folded,
            BetAction::Bet(x) | BetAction::Call(x) | BetAction::Raise(x) => BetStatus::In(x),
            BetAction::Check => unreachable!(),
        }
    }
}

impl BetStatus {
    /// Convience function so you don't have to match the enum.
    pub(crate) const fn is_folded(&self) -> bool {
        matches!(self, &BetStatus::Folded)
    }
}

impl Default for BetStatus {
    fn default() -> Self {
        BetStatus::Waiting
    }
}

#[derive(Debug)]
pub(crate) struct SeatedPlayers {
    players: [Option<SeatedPlayer>; MAX_PLAYERS],
    need_bets_from: Vec<PlayerId>,
    last_better: usize,
    pub dealer_token: usize,
    pub small_blind_token: usize,
    pub big_blind_token: usize,
}

impl Default for SeatedPlayers {
    fn default() -> Self {
        SeatedPlayers {
            //Apparently you can't do [None; MAX_PLAYERS] if the SeatedPlayer type doesn't
            //implement copy.
            players: [
                None, None, None, None, None, None, None, None, None, None, None, None,
            ],
            need_bets_from: Vec::new(),
            last_better: usize::MAX,
            dealer_token: usize::MAX,
            small_blind_token: usize::MAX,
            big_blind_token: usize::MAX,
        }
    }
}

impl SeatedPlayers {
    /// Place the given player into the given seat, and give them the given amount of money. Seat
    /// is an index (0-based).
    ///
    /// TODO abstract over Account struct?
    pub(crate) fn sit_down<C: Into<Currency>>(
        &mut self,
        aid: PlayerId,
        monies: C,
        seat: usize,
    ) -> Result<(), GameError> {
        if self.player_by_id(aid).is_some() {
            return Err(GameError::PlayerAlreadySeated);
        }
        if seat >= self.players.len() {
            return Err(GameError::InvalidSeat);
        }
        // The seat always exists, it's weather a player is sitting there we need to check
        match self.players[seat] {
            Some(_) => Err(GameError::SeatTaken),
            None => {
                self.players[seat] = Some(SeatedPlayer::new(aid, monies, seat));
                Ok(())
            }
        }
    }

    /// Removes the player from the table and returns the amount of money the person had.
    /// Parent is responsible for making sure the player can not leave mid round
    pub(crate) fn stand_up(&mut self, aid: PlayerId) -> Option<Currency> {
        let p = self.player_by_id(aid)?;
        let r = p.monies();
        self.players[p.seat_index] = None;
        Some(r)
    }

    /// Runs two bets, the small and big blind, and return the players and the bets they made
    pub(crate) fn blinds_bet<C: Into<Currency>>(
        &mut self,
        sb: C,
        bb: C,
    ) -> Result<(PlayerBetAction, PlayerBetAction), GameError> {
        let sbp = self
            .player_by_seat(self.small_blind_token)
            .ok_or(GameError::InvalidSeat)?
            .id;
        let bbp = self
            .player_by_seat(self.big_blind_token)
            .ok_or(GameError::InvalidSeat)?
            .id;
        let sb = sb.into();
        let bb = bb.into();
        let sba = self.bet(sbp, BetAction::Bet(sb), sb)?.0;
        let bba = self.bet(bbp, BetAction::Bet(bb), bb)?.0;
        // the blinds have bet, and we need to make sure they have the opportunity to bet again this
        // round, so rebuild need_bets_from
        self.need_bets_from = self
            .betting_players_iter_after(self.big_blind_token)
            .map(|sp| sp.id)
            .take(self.betting_players_count())
            .collect();
        self.need_bets_from.reverse();
        Ok(((sbp, sba), (bbp, bba)))
    }

    /// Informs us that a new round of betting is starting.
    ///
    /// We return an error if that shouldn't be the case, i.e. because we are missing bets from one
    /// or more players.
    pub(crate) fn next_betting_round(&mut self) -> Result<(), GameError> {
        if !self.need_bets_from.is_empty() {
            return Err(GameError::RoundNotOver);
        }
        for sp in self.betting_players_iter_mut() {
            sp.bet_status = BetStatus::Waiting;
        }
        self.need_bets_from = self
            .betting_players_iter_after(self.dealer_token)
            .map(|sp| sp.id)
            .take(self.betting_players_count())
            .collect();
        self.need_bets_from.reverse();
        Ok(())
    }

    /// The mutable version of `player_by_seat`
    fn _player_by_seat_mut(&mut self, n: usize) -> Option<&mut SeatedPlayer> {
        self.players_iter_mut().find(|x| x.seat_index == n)
    }

    /// Get a reference to the player in the given seat, if there is one
    fn player_by_seat(&self, n: usize) -> Option<&SeatedPlayer> {
        self.players_iter().find(|x| x.seat_index == n)
    }

    /// The mutable version of `player_by_id`
    pub(crate) fn player_by_id_mut(&mut self, player: PlayerId) -> Option<&mut SeatedPlayer> {
        self.players_iter_mut().find(|x| x.id == player)
    }

    /// Gets a reference to the player if their account ID could be found
    pub(crate) fn player_by_id(&self, player: PlayerId) -> Option<&SeatedPlayer> {
        self.players_iter().find(|x| x.id == player)
    }

    /// Check that the player can make the given bet, adjusting it if possible. Returns the
    /// (possibly adjusted) bet this player made and the bet amount that all players must meet
    /// (possibly adjusted).
    ///
    /// There is an unfortunate amount of stupidness in this code to make the borrow checker
    /// happy. Sorry.
    pub(crate) fn bet(
        &mut self,
        player: PlayerId,
        action: BetAction,
        current_bet: Currency,
    ) -> Result<(BetAction, Currency), BetError> {
        // Check player is even in the betting and that they're up next.
        // Stupidness here (getting the player_seat) because we don't want to maintain a borrow
        let player_seat = {
            let p = self.player_by_id(player).ok_or(BetError::PlayerNotFound)?;
            if !p.is_betting() {
                return Err(BetError::PlayerIsNotBetting);
            } else if self.need_bets_from.is_empty() {
                // perhaps the round should have been marked as ended?
                return Err(BetError::NoBetExpected);
            } else if self.need_bets_from[self.need_bets_from.len() - 1] != p.id {
                // the next player we expect a bet from is the last item in the list
                return Err(BetError::OutOfTurn);
            }
            p.seat_index
        };
        // Stupidness here (don't grab and keep the player reference) because we don't want to
        // maintain a borrow
        let ba = self
            .player_by_id_mut(player)
            .ok_or(BetError::PlayerNotFound)?
            .bet(action)?;
        // if the bet is for an amount greater than the current_bet, then we need to do a full
        // orbit around the table after this player to given everyone a chance to match it.
        // It'll be equal for calls. It'll be less for people going AllIn for less. It's more for
        // Raises (incl ALlIn Raises).
        match ba {
            BetAction::Check | BetAction::Fold => {
                self.need_bets_from.pop();
                Ok((ba, current_bet))
            }
            BetAction::Call(v) | BetAction::Bet(v) | BetAction::Raise(v) | BetAction::AllIn(v) => {
                match v.cmp(&current_bet) {
                    Ordering::Less => {
                        // If AllIn, this is ok. Otherwise it isn't. AllIn is the same as any other
                        // type of bet except in this case, hence this code organization.
                        if ba.is_allin() {
                            self.need_bets_from.pop();
                            Ok((ba, current_bet))
                        } else {
                            Err(BetError::BetTooLow)
                        }
                    }
                    Ordering::Equal => {
                        self.need_bets_from.pop();
                        Ok((ba, current_bet))
                    }
                    Ordering::Greater => {
                        // if this player just went all in, then there's one less betting player
                        // left than if this was a raise.
                        let n = if ba.is_allin() && self.betting_players_count() == 0 {
                            0
                        } else if ba.is_allin() {
                            self.betting_players_count()
                        } else {
                            self.betting_players_count() - 1
                        };
                        // yeah, the new bet is greater than old current bet, so need a bet from all
                        // players after this one at the table
                        self.need_bets_from = self
                            .betting_players_iter_after(player_seat)
                            .map(|sp| sp.id)
                            .take(n)
                            .collect();
                        self.need_bets_from.reverse();
                        Ok((ba, v))
                    }
                }
            }
        }
    }

    /// Returns an iterator over all seated players, preserving seat index
    fn players_iter(&self) -> impl Iterator<Item = &SeatedPlayer> + Clone + '_ {
        self.players
            .iter()
            .filter(|x| x.is_some())
            .map(|x| (x.as_ref().unwrap()))
    }

    /// Returns a mutable iterator over all seated players, preserving seat index
    fn players_iter_mut(&mut self) -> impl Iterator<Item = &mut SeatedPlayer> + '_ {
        self.players
            .iter_mut()
            .filter(|x| x.is_some())
            .map(|x| x.as_mut().unwrap())
    }

    /// Returns an iterator over players still in the betting, preserving seat index
    ///
    /// Note: say the only not-betting player is seat idx 2. This will list 0 and 1 before going
    /// on to 3 and the rest. This behavior is depended upon by betting_players_iter_after(...).
    fn betting_players_iter(&self) -> impl Iterator<Item = &SeatedPlayer> + Clone + '_ {
        self.players_iter().filter(|x| x.is_betting())
    }

    pub(crate) fn betting_players_count(&self) -> usize {
        self.betting_players_iter().count()
    }

    /// Returns an iterator over players still in the betting, preserving seat index
    ///
    /// Note: say the only not-betting player is seat idx 2. This will list 0 and 1 before going
    /// on to 3 and the rest.
    fn betting_players_iter_mut(&mut self) -> impl Iterator<Item = &mut SeatedPlayer> + '_ {
        self.players_iter_mut().filter(|x| x.is_betting())
    }

    /// Returns an iterator over the players in seat positions after the given seat index
    /// (0-indexed).
    ///
    /// Note that this will loop around the table up to almost twice. For example, given i=0, this
    /// will return an iterator over the seats starting at 1, through the end of the table, then
    /// start at 0 again and go through the end of the table. Only take the first few seats
    /// returned as you need them.
    fn betting_players_iter_after(
        &self,
        i: usize,
    ) -> impl Iterator<Item = &SeatedPlayer> + Clone + '_ {
        // Because rust will only let us return one type of iterator and we want to return early if
        // there are no betting players, we collect players into a vec and return an iterator over
        // that vec. Sucks.
        let last_betting_seat = match self.betting_players_iter().last() {
            None => return Vec::new().into_iter(),
            Some(sp) => sp.seat_index,
        };
        let si = if i >= last_betting_seat { 0 } else { i + 1 };
        self.betting_players_iter()
            .chain(self.betting_players_iter())
            .skip_while(move |x| x.seat_index < si)
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// All players that are still eligible to win some or all of the pot (i.e. not folded)
    pub(crate) fn eligible_players_iter(&self) -> impl Iterator<Item = &SeatedPlayer> + Clone + '_ {
        self.players_iter().filter(|x| !x.is_folded())
    }

    /// Verifies that the action has done a full orbit and that all players are in for the same
    /// amount (or all in, or out), thus the pot is ready to be finalized.
    ///
    /// AllIn players aren't "betting", so when iterating over all betting players, they are
    /// skipped. The only expected BetStatuses are In and Waiting.
    pub(crate) fn is_pot_ready(&self, current_bet: Currency) -> bool {
        // I believe this is the only check actually needed. If we're always given an acurate
        // current_bet in bet(...), then we'll always update need_bets_from. Thus, if the caller
        // calls bet(...) the right number of times, this should be enough.
        //
        // That said, in order to protect ourselves from a misbehaving caller (e.g. it ignored us
        // returning an error), we'll be super sure that we're done and make sure all betting
        // players meet the given current bet.
        if !self.need_bets_from.is_empty() {
            return false;
        }
        // The action has definitely orbited the table. Make sure everyone is in for the same
        // amount (or all in, or out)
        for player in self.betting_players_iter() {
            match player.bet_status {
                BetStatus::In(x) => {
                    if x != current_bet {
                        return false;
                    }
                }
                BetStatus::Waiting => return false,
                _ => unreachable!(),
            }
        }
        true
    }

    fn unfold_all(&mut self) {
        for player in self.players_iter_mut() {
            player.bet_status = BetStatus::Waiting;
            player.pocket = None;
        }
    }

    pub(crate) fn end_hand(
        &mut self,
        winnings: &HashMap<PlayerId, Currency>,
    ) -> Result<(), GameError> {
        for (player_id, amount) in winnings.iter() {
            if let Some(player) = self.player_by_id_mut(*player_id) {
                player.monies += *amount;
            }
            // TODO what about player ids that, for some reason, aren't known?
        }
        self.unfold_all();
        Ok(())
    }

    ///
    pub(crate) fn start_hand(&mut self) -> Result<(), GameError> {
        self.unfold_all();
        self.auto_fold_players();
        self.rotate_tokens()?;
        self.last_better = self.dealer_token;
        // prepare need_bets_from for the blinds bets
        self.need_bets_from = self
            .betting_players_iter_after(self.dealer_token)
            .map(|sp| sp.id)
            .take(self.betting_players_count())
            .collect();
        self.need_bets_from.reverse();
        Ok(())
    }

    fn auto_fold_players(&mut self) {
        for player in self.players_iter_mut() {
            if !player.has_monies() || player.auto_fold {
                player.bet_status = BetStatus::Folded;
            }
        }
    }

    fn rotate_tokens(&mut self) -> Result<(), GameError> {
        if self.betting_players_iter().count() < 2 {
            return Err(GameError::NotEnoughPlayers);
        }
        let mut s: [usize; 3] = [0, 0, 0];
        // iter borrows self, so have to work around borrowing rules
        // This might be fixable
        // Unwraps can't panic because iter size is at least 2 above, and `betting_players_iter_after` returns count * 2 entries, making a minimum values in the iter 4
        {
            let mut iter = self
                .betting_players_iter_after(self.dealer_token)
                .map(|x| x.seat_index);
            s[0] = iter.next().unwrap();
            s[1] = iter.next().unwrap();
            s[2] = iter.next().unwrap();
        }

        self.dealer_token = s[0];
        self.small_blind_token = s[1];
        // dealer_token can also be big blind
        self.big_blind_token = s[2];
        Ok(())
    }

    /// Takes a vector of cards and distributes them to the seated players.
    ///
    /// # Panics
    ///
    /// Panics if asked to deal a different number of pockets than players that are seated.
    ///
    /// # Returns
    ///
    /// HashMap with PlayerId keys and the cards they were dealt as values.
    pub(crate) fn deal_pockets(
        &mut self,
        mut pockets: Vec<[Card; 2]>,
    ) -> HashMap<PlayerId, [Card; 2]> {
        assert_eq!(pockets.len(), self.betting_players_iter().count());
        let dt = self.dealer_token;
        let mut ret = HashMap::new();
        // Can't use a betting_players_iter_after_mut() becasue can't chain/cycle mutable iterator
        // May be able to fix this with custom iterator
        // Until then, iterate wtice
        for player in self
            .betting_players_iter_mut()
            .skip_while(|x| x.seat_index < dt)
        {
            player.pocket = Some(pockets.pop().unwrap());
            ret.insert(player.id, player.pocket.unwrap());
        }
        for player in self
            .betting_players_iter_mut()
            .take_while(|x| x.seat_index < dt)
        {
            player.pocket = Some(pockets.pop().unwrap());
            ret.insert(player.id, player.pocket.unwrap());
        }
        ret
    }

    /// Returns the PlayerId of the next player we expect a bet from, or None if we don't expect a
    /// bet from anyone at this time.
    pub(crate) fn next_player(&self) -> Option<PlayerId> {
        match self.need_bets_from.is_empty() {
            true => None,
            false => Some(self.need_bets_from[self.need_bets_from.len() - 1]),
        }
    }
}

#[derive(Debug)]
pub(crate) struct SeatedPlayer {
    pub(crate) id: PlayerId,
    pub(crate) pocket: Option<[Card; 2]>,
    monies: Currency,
    bet_status: BetStatus,
    auto_fold: bool,
    pub(crate) seat_index: usize,
}

impl SeatedPlayer {
    /// This validates the user has enough money to make the given get
    /// It will concert bet() and call() into AllIn if required by user's stash
    pub(self) fn bet(&mut self, bet: BetAction) -> Result<BetAction, BetError> {
        if !self.has_monies() {
            return Err(BetError::HasNoMoney);
        }
        let existing_in = match self.bet_status {
            BetStatus::In(x) | BetStatus::AllIn(x) => x,
            BetStatus::Waiting => 0.into(),
            BetStatus::Folded => unreachable!(),
        };
        let r = match bet {
            BetAction::Bet(x) | BetAction::Call(x) | BetAction::Raise(x) => {
                if x < existing_in {
                    // Can't bet less than existing bet. Rememeber, seeing Call(10), Call(20) from
                    // the same player means the player means they want to be in for a total of 20,
                    // not 30.
                    return Err(BetError::BetTooLow);
                }
                let additional_in = x - existing_in;
                match self.monies.cmp(&additional_in) {
                    Ordering::Less => {
                        // Only called when blinds are short stacked.
                        let r = BetAction::AllIn(self.monies + existing_in);
                        self.monies = 0.into();
                        r
                    }
                    _ => {
                        self.monies -= additional_in;
                        bet
                    }
                }
            }
            BetAction::AllIn(x) => {
                if x < existing_in {
                    // Can't bet less than existing bet. Rememeber, seeing Call(10), Call(20) from
                    // the same player means the player means they want to be in for a total of 20,
                    // not 30.
                    return Err(BetError::BetTooLow);
                }
                let additional_in = x - existing_in;
                if additional_in != self.monies {
                    return Err(BetError::AllInWithoutBeingAllIn);
                }
                self.monies = 0.into();
                bet
            }
            BetAction::Check => match self.bet_status {
                // check with no current bet from us means we're in for 0 (e.g. post flop first to
                // act)
                BetStatus::Waiting => BetAction::Bet(0.into()),
                // check with a current bet means we're the big blind preflop (hopefully, else bug)
                BetStatus::In(x) => BetAction::Bet(x),
                BetStatus::Folded | BetStatus::AllIn(_) => unreachable!(),
            },
            BetAction::Fold => bet,
        };
        self.bet_status = BetStatus::from(r);
        Ok(r)
    }

    pub(self) fn new<C: Into<Currency>>(id: PlayerId, monies: C, seat_index: usize) -> Self {
        SeatedPlayer {
            id,
            pocket: None,
            monies: monies.into(),
            bet_status: BetStatus::Folded,
            auto_fold: false,
            seat_index,
        }
    }
    pub(crate) const fn bet_status(&self) -> BetStatus {
        self.bet_status
    }
    pub(crate) const fn monies(&self) -> Currency {
        self.monies
    }
    pub(crate) fn has_monies(&self) -> bool {
        self.monies > 0.into()
    }

    pub(crate) const fn is_folded(&self) -> bool {
        self.bet_status.is_folded()
    }

    /// Returns true if player is still in the betting
    /// Notably, `all_in` players are no longer betting, and excluded
    pub(crate) const fn is_betting(&self) -> bool {
        matches!(self.bet_status, BetStatus::In(_) | BetStatus::Waiting)
    }
}

#[cfg(test)]
mod tests {
    use super::super::deck::Deck;
    use super::*;

    #[test]
    fn token_rotation() {
        let mut sp = SeatedPlayers::default();
        sp.sit_down(1, 10, 0).unwrap();
        sp.sit_down(2, 10, 11).unwrap();
        sp.start_hand().unwrap();
        assert_eq!(sp.dealer_token, 0);
        assert_eq!(sp.small_blind_token, 11);
        assert_eq!(sp.big_blind_token, 0);

        let mut sp = SeatedPlayers::default();
        sp.sit_down(1, 10, 0).unwrap();
        sp.sit_down(2, 10, 1).unwrap();
        sp.sit_down(3, 0, 11).unwrap();
        sp.start_hand().unwrap();
        assert_eq!(sp.dealer_token, 0);
        assert_eq!(sp.small_blind_token, 1);
        assert_eq!(sp.big_blind_token, 0);

        let mut sp = SeatedPlayers::default();
        sp.sit_down(1, 10, 0).unwrap();
        sp.sit_down(2, 10, 3).unwrap();
        sp.sit_down(3, 10, 5).unwrap();
        sp.sit_down(4, 10, 7).unwrap();
        sp.sit_down(5, 10, 11).unwrap();
        sp.start_hand().unwrap();
        assert_eq!(sp.dealer_token, 0);
        assert_eq!(sp.small_blind_token, 3);
        assert_eq!(sp.big_blind_token, 5);
        sp.end_hand(&HashMap::new()).unwrap();
        sp.start_hand().unwrap();
        assert_eq!(sp.dealer_token, 3);
        assert_eq!(sp.small_blind_token, 5);
        assert_eq!(sp.big_blind_token, 7);
    }

    #[test]
    fn all_in_on_blind() {
        let mut sp = SeatedPlayers::default();
        sp.sit_down(1, 2, 0).unwrap();
        sp.sit_down(2, 10, sp.players.len() - 1).unwrap();
        sp.start_hand().unwrap();
        sp.blinds_bet(5, 10).unwrap();
        assert_eq!(
            sp.player_by_id(1).unwrap().bet_status,
            BetStatus::AllIn(2.into())
        );
        assert_eq!(
            sp.player_by_id(2).unwrap().bet_status,
            BetStatus::In(5.into())
        );
    }

    #[test]
    fn player_cant_sit_twice() {
        let mut sp = SeatedPlayers::default();
        sp.sit_down(1, 10, 0).unwrap();
        let r = sp.sit_down(1, 10, 1);
        assert!(r.is_err());
    }

    #[test]
    fn seat_taken() {
        let mut sp = SeatedPlayers::default();
        sp.sit_down(1, 10, 0).unwrap();
        let r = sp.sit_down(2, 10, 0);
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
                let mut sp = SeatedPlayers::default();
                for seat in 0..n_players {
                    sp.sit_down(seat as PlayerId, 100, seat as usize).unwrap();
                }
                // move dealer token to correct player
                while sp.dealer_token != first as usize {
                    sp.start_hand().unwrap();
                }
                let mut deck = Deck::default();
                let pockets = deck.deal_pockets(n_players as u8).unwrap();
                // this is the actual test. Does this panic?
                sp.deal_pockets(pockets);
                // okay so it didn't. let's make sure every player has a pocket.
                for player in sp.players_iter() {
                    assert!(player.pocket.is_some());
                }
            }
        }
    }

    /// betting_players_iter_after still returns the right number of players, regardless of the seat
    /// index given to it. They're also in the right order.
    #[test]
    fn betting_players_iter_after() {
        for given in 0..=3usize {
            let mut sp = SeatedPlayers::default();
            for seat in 0..=3usize {
                sp.sit_down(seat as PlayerId, 100, seat).unwrap();
            }
            sp.start_hand().unwrap();
            let v: Vec<_> = sp
                .betting_players_iter_after(given)
                .map(|sp| sp.id)
                .take(4)
                .collect();
            match given {
                0 => assert_eq!(v, vec![1, 2, 3, 0]),
                1 => assert_eq!(v, vec![2, 3, 0, 1]),
                2 => assert_eq!(v, vec![3, 0, 1, 2]),
                3 => assert_eq!(v, vec![0, 1, 2, 3]),
                _ => unreachable!(),
            }
        }
    }
}
