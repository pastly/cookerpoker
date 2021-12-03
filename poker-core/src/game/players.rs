use super::{deck::Card, BetAction, BetError, GameError};
pub const MAX_PLAYERS: usize = 12;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BetStatus {
    Folded,
    Waiting,
    In(i32),
    AllIn(i32),
}

impl From<BetAction> for BetStatus {
    fn from(ba: BetAction) -> Self {
        match ba {
            BetAction::AllIn(x) => BetStatus::AllIn(x),
            BetAction::Fold => BetStatus::Folded,
            BetAction::Bet(x) | BetAction::Call(x) => BetStatus::In(x),
            BetAction::Check => unreachable!(),
        }
    }
}

impl BetStatus {
    /// Convience function so you don't have to match the enum.
    pub fn is_folded(&self) -> bool {
        matches!(self, &BetStatus::Folded)
    }
}

impl Default for BetStatus {
    fn default() -> Self {
        BetStatus::Waiting
    }
}

#[derive(Debug)]
pub struct SeatedPlayers {
    players: [Option<SeatedPlayer>; MAX_PLAYERS],
    last_better: usize,
    pub dealer_token: usize,
    pub small_blind_token: usize,
    pub big_blind_token: usize,
}

impl Default for SeatedPlayers {
    fn default() -> Self {
        SeatedPlayers {
            //Apparently you can't do [None; MAX_PLAYERS] if the SeatedPlayer type doesn't implement copy.
            players: [
                None, None, None, None, None, None, None, None, None, None, None, None,
            ],
            last_better: usize::MAX,
            dealer_token: usize::MAX,
            small_blind_token: usize::MAX,
            big_blind_token: usize::MAX,
        }
    }
}

impl SeatedPlayers {
    /// TODO abstract over Account struct?
    pub fn sit_down(&mut self, aid: i32, monies: i32, seat: usize) -> Result<(), GameError> {
        if self.player_by_id(aid).is_some() {
            return Err(GameError::PlayerAlreadySeated);
        }
        if seat >= MAX_PLAYERS {
            return Err(GameError::InvalidSeat);
        }
        // The seat always exists, it's weather a player is sitting there we need to check
        match self.players[seat] {
            Some(_) => Err(GameError::SeatTaken),
            None => {
                self.players[seat] = Some(SeatedPlayer::new(aid, monies));
                Ok(())
            }
        }
    }
    /// Moves betting round forward and returns account id of next better
    fn next_better(&mut self) -> i32 {
        let (i, aid) = self
            .betting_players_iter_after(self.last_better)
            .map(|(i, x)| (i, x.id))
            .next()
            .unwrap();
        self.last_better = i;
        aid
    }

    /// Runs two bets, the blinds
    /// Needed in this struct because next_better is private
    pub fn blinds_bet(&mut self, sb: i32, bb: i32) -> Result<i32, BetError> {
        let sbp = self.next_better();
        let bbp = self.bet(sbp, BetAction::Bet(sb))?;
        let r = self.bet(bbp, BetAction::Bet(bb))?;
        Ok(r)
    }

    pub fn player_by_id(&mut self, player: i32) -> Option<&mut SeatedPlayer> {
        self.players_iter_mut()
            .map(|(_, x)| x)
            .find(|x| x.id == player)
    }

    /// This function is not aware of the current bet. As such validation must be handled before this function:
    /// * Check's should be converted to Calls
    /// * Validation that the bet meets the current bet amount
    ///
    /// Returns the account id of the next better.
    pub fn bet(&mut self, player: i32, action: BetAction) -> Result<i32, BetError> {
        // Check player is even in the betting
        let p: &mut SeatedPlayer = self.player_by_id(player).ok_or(BetError::PlayerNotFound)?;
        if !p.is_betting() {
            return Err(BetError::PlayerIsNotBetting);
        }
        // Call player.bet()
        p.bet(action)?;

        // Move the betting round forward
        let nb = self.next_better();

        // Return
        Ok(nb)
    }

    /// Returns an iterator over all seated players, preserving seat index
    pub fn players_iter(&self) -> impl Iterator<Item = (usize, &SeatedPlayer)> + Clone + '_ {
        self.players
            .iter()
            .enumerate()
            .filter(|(_, y)| y.is_some())
            .map(|(x, y)| (x, y.as_ref().unwrap()))
    }

    /// Returns a mutable iterator over all seated players, preserving seat index
    pub fn players_iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut SeatedPlayer)> + '_ {
        self.players
            .iter_mut()
            .enumerate()
            .filter(|(_, y)| y.is_some())
            .map(|(x, y)| (x, y.as_mut().unwrap()))
    }

    /// Returns an iterator over players still in the betting, preserving seat index
    pub fn betting_players_iter(
        &self,
    ) -> impl Iterator<Item = (usize, &SeatedPlayer)> + Clone + '_ {
        self.players_iter().filter(|(_, y)| y.is_betting())
    }

    pub fn betting_players_iter_after(
        &self,
        i: usize,
    ) -> impl Iterator<Item = (usize, &SeatedPlayer)> + Clone + '_ {
        let si = if i >= MAX_PLAYERS - 1 { 0 } else { i + 1 };
        self.betting_players_iter()
            .chain(self.betting_players_iter())
            .skip_while(move |(x, _)| x < &si)
    }

    /// Checks all seated players `BetStatus` and validates that the pot is ready to be finalized
    pub fn pot_is_right(&self, current_bet: i32) -> bool {
        for (_, player) in self.betting_players_iter() {
            match player.bet_status {
                BetStatus::In(x) => {
                    if x == current_bet {
                        continue;
                    }
                    return false;
                }
                BetStatus::Waiting => return false,
                _ => unreachable!(),
            }
        }
        true
    }

    fn unfold_all(&mut self) {
        for (_, player) in self.players_iter_mut() {
            player.bet_status = BetStatus::Waiting;
        }
    }

    pub fn end_hand(&mut self) -> Result<(), GameError> {
        self.unfold_all();
        Ok(())
    }

    ///
    pub fn start_hand(&mut self) -> Result<Vec<i32>, GameError> {
        self.unfold_all();
        self.auto_fold_players();
        self.rotate_tokens()?;
        self.last_better = self.dealer_token;
        Ok(self.betting_players_iter().map(|(_, y)| y.id).collect())
    }

    fn auto_fold_players(&mut self) {
        for (_, player) in self.players_iter_mut() {
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
        // Unwraps can't because mis size is 2 above, and after returns count * 2 entries, making a minimum values in the iter 4
        {
            let od = self.dealer_token;
            dbg!(&od);
            let mut iter = self.betting_players_iter_after(od).map(|(x, _)| x);
            s[0] = iter.next().unwrap();
            s[1] = iter.next().unwrap();
            s[2] = iter.next().unwrap();
        }
        dbg!(&s);
        self.dealer_token = s[0];
        // dealer_token can also be big blind
        self.small_blind_token = s[1];
        self.big_blind_token = s[2];
        Ok(())
    }
}

#[derive(Debug)]
pub struct SeatedPlayer {
    pub id: i32,
    pub pocket: Option<[Card; 2]>,
    monies: i32,
    pub bet_status: BetStatus,
    pub auto_fold: bool,
}

impl SeatedPlayer {
    /// This validates the user has enough money to make the given get
    /// It will concert bet() and call() into AllIn if required by user's stash
    pub fn bet(&mut self, bet: BetAction) -> Result<BetAction, BetError> {
        use std::cmp::Ordering;
        if !self.has_monies() {
            return Err(BetError::HasNoMoney);
        }
        let r = match bet {
            BetAction::Bet(x) | BetAction::Call(x) => match self.monies.cmp(&x) {
                Ordering::Less => {
                    // Only called when blinds are short stacked.
                    let r = BetAction::AllIn(self.monies);
                    self.monies = 0;
                    r
                }
                _ => {
                    self.monies -= x;
                    bet
                }
            },
            BetAction::AllIn(x) => {
                if x != self.monies {
                    return Err(BetError::AllInWithoutBeingAllIn);
                }
                self.monies = 0;
                BetAction::AllIn(self.monies)
            }
            BetAction::Check => unimplemented!(),
            _ => bet,
        };
        self.bet_status = BetStatus::from(r);
        Ok(r)
    }

    pub fn new(id: i32, monies: i32) -> Self {
        SeatedPlayer {
            id,
            pocket: None,
            monies,
            bet_status: BetStatus::Folded,
            auto_fold: false,
        }
    }
    pub fn has_monies(&self) -> bool {
        self.monies > 0
    }

    pub fn is_folded(&self) -> bool {
        self.bet_status.is_folded()
    }

    /// Returns true is player is still in the betting
    /// Notably, `all_in` players are no longer better, and excluded
    pub fn is_betting(&self) -> bool {
        matches!(self.bet_status, BetStatus::In(_) | BetStatus::Waiting)
    }
}

#[cfg(test)]
mod tests {
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
        sp.end_hand();
        sp.start_hand().unwrap();
        assert_eq!(sp.dealer_token, 3);
        assert_eq!(sp.small_blind_token, 5);
        assert_eq!(sp.big_blind_token, 7);
    }

    #[test]
    fn all_in_on_blind() {
        let mut sp = SeatedPlayers::default();
        sp.sit_down(1, 2, 0).unwrap();
        sp.sit_down(2, 10, MAX_PLAYERS - 1).unwrap();
        sp.start_hand().unwrap();
        sp.blinds_bet(5, 10).unwrap();
        assert_eq!(sp.player_by_id(1).unwrap().bet_status, BetStatus::AllIn(2));
        assert_eq!(sp.player_by_id(2).unwrap().bet_status, BetStatus::In(5));
    }

    #[test]
    fn player_cant_sit_twice() {
        let mut sp = SeatedPlayers::default();
        sp.sit_down(1, 10, 0).unwrap();
        let r = sp.sit_down(1, 10, 1);
        assert!(r.is_err());
    }
}
