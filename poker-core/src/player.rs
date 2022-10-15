use crate::bet::{BetAction, BetStatus};
use crate::deck::Card;
use crate::GameError;
use crate::{Currency, PlayerId, MAX_PLAYERS};
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const POCKET_SIZE: usize = 2;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Players {
    pub(crate) players: [Option<Player>; MAX_PLAYERS],
    /// loation of dealer token, index into players array
    pub token_dealer: usize,
    /// loation of small blind token, index into players array
    pub token_sb: usize,
    /// loation of big blind token, index into players array
    pub token_bb: usize,
    /// players (as indexes into players array that we need bets from next, ordered in reverse
    /// (next expected better is last in this Vec, and so on)
    pub(crate) need_bets_from: Vec<usize>,
}

impl Default for Players {
    fn default() -> Self {
        Self {
            players: [None; MAX_PLAYERS],
            token_dealer: 0,
            token_sb: 0,
            token_bb: 0,
            need_bets_from: Vec::with_capacity(MAX_PLAYERS),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub stack: Currency,
    pub pocket: Option<[Card; POCKET_SIZE]>,
    pub bet_status: BetStatus,
    /// Whether or not this player wants to be dealt in. They could be forced to sit out if they get
    /// stacked.
    pub sitting_out: bool,
}
impl Players {
    pub fn player_by_id(&self, id: PlayerId) -> Option<&Player> {
        self.players_iter().find(|x| x.id == id)
    }

    pub(crate) fn player_with_index_by_id(&self, id: PlayerId) -> Option<(usize, &Player)> {
        self.players_iter_with_index().find(|(_, x)| x.id == id)
    }

    pub(crate) fn player_by_id_mut(&mut self, id: PlayerId) -> Option<&mut Player> {
        self.players_iter_mut().find(|x| x.id == id)
    }

    pub(crate) fn seat_player(&mut self, player: Player) -> Result<usize, GameError> {
        if let Some(seat_idx) = self.next_empty_seat() {
            self.players[seat_idx] = Some(player);
            Ok(seat_idx)
        } else {
            Err(GameError::TableFull)
        }
    }

    pub(crate) fn deal_pockets(&mut self, mut pockets: Vec<[Card; 2]>) {
        assert_eq!(pockets.len(), self.betting_players_count());
        let dt = self.token_dealer;
        // Can't use a betting_players_iter_after_mut() becasue can't chain/cycle mutable iterator
        // May be able to fix this with custom iterator
        // Until then, iterate twice
        for (_, player) in self.betting_players_iter_mut().skip_while(|(i, _)| *i < dt) {
            player.pocket = Some(pockets.pop().unwrap());
        }
        for (_, player) in self.betting_players_iter_mut().take_while(|(i, _)| *i < dt) {
            player.pocket = Some(pockets.pop().unwrap());
        }
    }

    fn next_empty_seat(&self) -> Option<usize> {
        self.players
            .iter()
            .enumerate()
            .find(|(_idx, p)| p.is_none())
            .map(|(i, _)| i)
    }

    pub fn players_iter(&self) -> impl Iterator<Item = &Player> /*+ Clone + '_ */ {
        self.players.iter().filter_map(|x| x.as_ref())
    }

    fn players_iter_mut(&mut self) -> impl Iterator<Item = &mut Player> {
        self.players.iter_mut().filter_map(|x| x.as_mut())
    }

    /// Iterate over all players, returning their index into the player array as well
    pub fn players_iter_with_index(&self) -> impl Iterator<Item = (usize, &Player)> {
        self.players
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_some())
            .map(|(i, x)| (i, x.as_ref().unwrap()))
    }

    /// Iterate over all players, returning their index into the player array as well
    fn players_iter_mut_with_index(&mut self) -> impl Iterator<Item = (usize, &mut Player)> {
        self.players
            .iter_mut()
            .enumerate()
            .filter(|(_, x)| x.is_some())
            .map(|(i, x)| (i, x.as_mut().unwrap()))
    }

    /// Like betting_players_iter, but with the seat index for each player also returned
    /// Returns an iterator over players still in the betting and their seat index
    ///
    /// Note: say the only not-betting player is seat idx 2. This will list 0 and 1 before going
    /// on to 3 and the rest. This behavior is depended upon by betting_players_iter_after(...).
    fn betting_players_iter(&self) -> impl Iterator<Item = (usize, &Player)> /*+ Clone + '_*/ {
        self.players_iter_with_index()
            .filter(|(_, x)| x.is_betting())
    }

    pub(crate) fn betting_players_count(&self) -> usize {
        self.betting_players_iter().count()
    }

    fn betting_players_iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut Player)> {
        self.players_iter_mut_with_index()
            .filter(|(_, x)| x.is_betting())
    }

    /// Returns an iterator over the players in seat positions after the given seat index
    /// (0-indexed).
    ///
    /// Note that this will loop around the table up to almost twice. For example, given i=0, this
    /// will return an iterator over the seats starting at 1, through the end of the table, then
    /// start at 0 again and go through the end of the table. Only take the first few seats
    /// returned as you need them.
    pub(crate) fn betting_players_iter_after(
        &self,
        i: usize,
    ) -> impl Iterator<Item = (usize, &Player)> /*+ Clone + '_*/ {
        // Because rust will only let us return one type of iterator and we want to return early if
        // there are no betting players, we collect players into a vec and return an iterator over
        // that vec. Sucks.
        let last_betting_seat = match self.betting_players_iter().last() {
            None => return Vec::new().into_iter(),
            Some((i, _)) => i,
        };
        let si = if i >= last_betting_seat { 0 } else { i + 1 };
        self.betting_players_iter()
            .chain(self.betting_players_iter())
            .skip_while(move |(i, _)| *i < si)
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// All players that are still eligible to win some or all of the pot (i.e. not folded)
    pub(crate) fn eligible_players_iter(&self) -> impl Iterator<Item = &Player> /*+ Clone + '_*/ {
        self.players_iter().filter(|x| !x.is_folded())
    }

    fn seated_players_iter(&self) -> impl Iterator<Item = &Player> {
        self.players_iter().filter(|x| !x.sitting_out)
    }

    pub(crate) fn clean_state(&mut self) {
        for p in self.players_iter_mut() {
            p.bet_status = BetStatus::Waiting;
            p.pocket = None;
        }
    }

    fn auto_sitout(&mut self) {
        for p in self.players_iter_mut() {
            if p.stack < 1 {
                p.sitting_out = true;
            }
        }
    }

    pub(crate) fn start_hand(&mut self) -> Result<(), GameError> {
        self.auto_sitout();
        if self.seated_players_iter().count() < 2 {
            return Err(GameError::NotEnoughPlayers);
        }
        //self.unfold_all();
        //self.auto_fold_players();
        for p in self.players_iter_mut() {
            p.bet_status = BetStatus::Waiting;
            p.pocket = None;
        }
        self.rotate_tokens()?;
        //self.last_better = self.token_dealer;
        // prepare need_bets_from for the blinds bets
        self.need_bets_from = self
            .betting_players_iter_after(self.token_dealer)
            .map(|(i, _)| i)
            .take(self.betting_players_count())
            .collect();
        self.need_bets_from.reverse();
        Ok(())
    }

    pub(crate) fn end_hand(
        &mut self,
        winnings: &HashMap<PlayerId, Currency>,
    ) -> Result<(), GameError> {
        for (player_id, amount) in winnings.iter() {
            if let Some(player) = self.player_by_id_mut(*player_id) {
                player.stack += *amount;
            }
            // TODO: what about player IDs that are unknown for some reason?
        }
        //self.unfold_all();
        Ok(())
    }

    /// Informs us that the next street is beginning so we can reinit state if needed
    ///
    /// We return an error if we don't think the next street should be starting at this point.
    pub(crate) fn next_street(&mut self) -> Result<(), GameError> {
        if !self.need_bets_from.is_empty() {
            return Err(GameError::StreetNotComplete);
        }
        for (_, p) in self.betting_players_iter_mut() {
            p.bet_status = BetStatus::Waiting;
        }
        self.need_bets_from = self
            .betting_players_iter_after(self.token_dealer)
            .map(|(i, _)| i)
            .take(self.betting_players_count())
            .collect();
        self.need_bets_from.reverse();
        Ok(())
    }

    pub(crate) fn rotate_tokens(&mut self) -> Result<(), GameError> {
        if self.betting_players_iter().count() < 2 {
            return Err(GameError::NotEnoughPlayers);
        }
        let mut s: [usize; 3] = [0, 0, 0];
        // iter borrows self, so have to work around borrowing rules
        // This might be fixable
        // Unwraps can't panic because iter size is at least 2 above, and `betting_players_iter_after` returns count * 2 entries, making a minimum values in the iter 4
        {
            let mut iter = self
                .betting_players_iter_after(self.token_dealer)
                .map(|(i, _)| i);
            s[0] = iter.next().unwrap();
            s[1] = iter.next().unwrap();
            s[2] = iter.next().unwrap();
        }

        self.token_dealer = s[0];
        self.token_sb = s[1];
        // token_dealer can also be big blind
        self.token_bb = s[2];
        Ok(())
    }
}

impl Player {
    pub(crate) fn new(id: PlayerId, stack: Currency) -> Self {
        Self {
            id,
            stack,
            pocket: None,
            bet_status: BetStatus::Waiting,
            sitting_out: stack < 1,
        }
    }

    /// Returns true if player is still in the betting
    /// Notably, `all_in` players are no longer betting, and excluded
    pub(crate) const fn is_betting(&self) -> bool {
        matches!(self.bet_status, BetStatus::In(_) | BetStatus::Waiting)
    }

    pub(crate) const fn is_folded(&self) -> bool {
        matches!(self.bet_status, BetStatus::Folded)
    }

    /// Validates that the player has enough money to make the given bet.
    /// Coerces bet/call into allin if required by player's stack.
    /// Updates player's stack.
    pub(crate) fn bet(&mut self, bet: BetAction) -> Result<BetAction, GameError> {
        if self.stack <= 0 {
            return Err(GameError::PlayerStackTooShort);
        }
        let existing_in = match self.bet_status {
            BetStatus::In(x) | BetStatus::AllIn(x) => x,
            BetStatus::Waiting => 0,
            BetStatus::Folded => unreachable!(),
        };
        let return_bet = match bet {
            BetAction::Fold => bet,
            BetAction::Check => match self.bet_status {
                // check with a current bet means we're the big blind preflop (hopefully, else bug)
                BetStatus::In(x) => BetAction::Bet(x),
                BetStatus::Waiting => BetAction::Check,
                BetStatus::Folded | BetStatus::AllIn(_) => unreachable!(),
            },
            BetAction::Bet(x) | BetAction::Call(x) | BetAction::Raise(x) => {
                if x < existing_in {
                    // Can't bet less than existing bet. Rememeber, seeing Call(10), Call(20) from
                    // the same player means the player means they want to be in for a total of 20,
                    // not 30.
                    return Err(GameError::InvalidBet);
                }
                let additional_in = x - existing_in;
                match self.stack.cmp(&additional_in) {
                    Ordering::Less => {
                        // Only called when blinds are short stacked.
                        let r = BetAction::AllIn(self.stack + existing_in);
                        self.stack = 0;
                        r
                    }
                    _ => {
                        self.stack -= additional_in;
                        bet
                    }
                }
            }
            BetAction::AllIn(x) => {
                if x < existing_in {
                    // Can't bet less than existing bet. Rememeber, seeing Call(10), Call(20) from
                    // the same player means the player means they want to be in for a total of 20,
                    // not 30.
                    return Err(GameError::InvalidBet);
                }
                let additional_in = x - existing_in;
                if additional_in != self.stack {
                    return Err(GameError::InvalidBet);
                }
                self.stack = 0;
                bet
            }
        };
        self.bet_status = BetStatus::from(return_bet);
        Ok(return_bet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_rotation() {
        let mut players = Players::default();
        const LAST_SEAT: usize = MAX_PLAYERS - 1;
        players.players[0] = Some(Player::new(1, 10));
        players.players[LAST_SEAT] = Some(Player::new(2, 10));
        players.rotate_tokens().unwrap();
        assert_eq!(players.token_dealer, LAST_SEAT);
        assert_eq!(players.token_sb, 0);
        assert_eq!(players.token_bb, LAST_SEAT);
        players.rotate_tokens().unwrap();
        assert_eq!(players.token_dealer, 0);
        assert_eq!(players.token_sb, LAST_SEAT);
        assert_eq!(players.token_bb, 0);

        // TODO: test that empty stack players are skipped over
        // let mut players = Players::default();
        // players.players[0] = Some(Player::new(1, 10));
        // players.players[1] = Some(Player::new(2, 10));
        // players.players[LAST_SEAT] = Some(Player::new(3, 0));
        // assert_eq!(players.token_dealer, 0);
        // assert_eq!(players.token_sb, 1);
        // assert_eq!(players.token_bb, 0);

        let mut players = Players::default();
        players.players[0] = Some(Player::new(1, 10));
        players.players[3] = Some(Player::new(2, 10));
        players.players[5] = Some(Player::new(3, 10));
        players.players[7] = Some(Player::new(4, 10));
        players.players[LAST_SEAT] = Some(Player::new(5, 10));
        players.rotate_tokens().unwrap();
        assert_eq!(players.token_dealer, 3);
        assert_eq!(players.token_sb, 5);
        assert_eq!(players.token_bb, 7);
        players.rotate_tokens().unwrap();
        assert_eq!(players.token_dealer, 5);
        assert_eq!(players.token_sb, 7);
        assert_eq!(players.token_bb, LAST_SEAT);
        players.rotate_tokens().unwrap();
        assert_eq!(players.token_dealer, 7);
        assert_eq!(players.token_sb, LAST_SEAT);
        assert_eq!(players.token_bb, 0);
        players.rotate_tokens().unwrap();
        assert_eq!(players.token_dealer, LAST_SEAT);
        assert_eq!(players.token_sb, 0);
        assert_eq!(players.token_bb, 3);
        players.rotate_tokens().unwrap();
        assert_eq!(players.token_dealer, 0);
        assert_eq!(players.token_sb, 3);
        assert_eq!(players.token_bb, 5);
        players.rotate_tokens().unwrap();
        assert_eq!(players.token_dealer, 3);
        assert_eq!(players.token_sb, 5);
        assert_eq!(players.token_bb, 7);
    }

    // betting_players_iter_after still returns the right number of players, regardless of the seat
    // index given to it. They're also in the right order.
    #[test]
    fn betting_players_iter_after() {
        for given in 0..=3usize {
            let mut players = Players::default();
            for seat in 0..=3usize {
                players.players[seat] = Some(Player::new(seat as PlayerId, 100));
            }
            let v: Vec<_> = players
                .betting_players_iter_after(given)
                .map(|(_, p)| p.id)
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
