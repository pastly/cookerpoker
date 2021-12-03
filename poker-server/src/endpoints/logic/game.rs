#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(clippy::unused_unit)]
#![allow(unused_imports)]
use super::*;
use poker_core::{
    deck::{self, Card, Deck, Rank, Suit},
    hand::*,
};
use std::collections::HashMap;
use table::TableType;

const MAX_PLAYERS: usize = 12;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum BetAction {
    Check,
    Fold,
    Call(i32),
    Bet(i32),
    AllIn(i32),
}

#[derive(Debug, PartialEq)]
pub enum BetStatus {
    Folded,
    Waiting,
    In(i32),
    AllIn(i32),
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
pub enum GameState {
    Dealing,
    Betting(BetRound),
    Winner(i32, i32),
    WinnerDuringBet(i32, i32),
}

#[derive(Debug)]
pub enum BetRound {
    PreFlop(i32),
    Flop(i32),
    Turn(i32),
    River(i32),
}

#[derive(Debug)]
pub struct GameInProgress {
    table_type: TableType,
    pub table_cards: [Option<Card>; 5],
    pub seated_players: SeatedPlayers,
    pub pots: Pot,
    pub state: GameState,
    pub small_blind: i32,
    d: Deck,
}

/*impl GameInProgress {
    fn start_round(&mut self) -> Result<(), GameError> {
        self.state = GameState::Dealing;
        self.d = Deck::new();
        let seated_players = self.seated_players.start_hand(self.small_blind)?;
        self.pots = self.pots.start_hand(seated_players);
        self.blinds_bet()?;
        let np = self.seated_players.num_players_still_in();
        let pockets = self.d.deal_pockets(np)?;

        unimplemented!()
    }

    fn finalize_hand(&mut self) -> Result<GameState, GameError> {
        self.seated_players.end_hand()?;
        // TODO Fold 'auto-fold' players?
        // TODO Force rocket to update DB? Probably by returning State enum?
        unimplemented!()
    }

    fn blinds_bet(&mut self) -> Result<i32, GameError> {}

}*/

#[derive(Debug)]
pub struct SeatedPlayers {
    players: [Option<SeatedPlayer>; MAX_PLAYERS],
    last_better: usize,
    dealer_token: usize,
    small_blind_token: usize,
    big_blind_token: usize,
}

impl Default for SeatedPlayers {
    fn default() -> Self {
        SeatedPlayers {
            //Apparently you can't do [None; MAX_PLAYERS] if the SeatedPlayer type doesn't implement copy.
            players: [
                None, None, None, None, None, None, None, None, None, None, None, None,
            ],
            last_better: 0,
            dealer_token: 0,
            small_blind_token: 1,
            big_blind_token: 2,
        }
    }
}

impl SeatedPlayers {
    /// Moves betting round forward and returns account id of next better
    /// Returns None if betting round is over.
    fn next_better(&mut self, current_bet: i32) -> Option<i32> {
        if self.pot_is_right(current_bet) || self.betting_players_iter().count() == 1 {
            None
        } else {
            //TODO this doesn't work
            //Need to find self.last_better's place in self.betting_players_iteR() (even if they are no longer in it)
            //Then call .cycle() on iter
            //Then .next()
            //Assert new better is not the same as the old
            let ov = if self.last_better == MAX_PLAYERS - 1 {
                0
            } else {
                self.last_better
            };
            let (i, aid) = self
                .betting_players_iter()
                .map(|(i, x)| (i, x.id))
                .skip_while(|(i, _)| i <= &ov)
                .cycle()
                .next()
                .unwrap();
            self.last_better = i;
            Some(aid)
        }
    }

    pub fn bet(&mut self, player: i32, action: BetAction) -> Result<i32, BetError> {
        unimplemented!()
    }

    /// Returns an iterator over all seated players, preserving seat index
    pub fn players_iter(&self) -> impl Iterator<Item = (usize, &SeatedPlayer)> + Clone + '_ {
        self.players
            .iter()
            .enumerate()
            .filter(|(x, y)| y.is_some())
            .map(|(x, y)| (x, y.as_ref().unwrap()))
    }

    /// Returns a mutable iterator over all seated players, preserving seat index
    pub fn players_iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut SeatedPlayer)> + '_ {
        self.players
            .iter_mut()
            .enumerate()
            .filter(|(x, y)| y.is_some())
            .map(|(x, y)| (x, y.as_mut().unwrap()))
    }

    /// Returns an iterator over players still in the betting, preserving seat index
    pub fn betting_players_iter(
        &self,
    ) -> impl Iterator<Item = (usize, &SeatedPlayer)> + Clone + '_ {
        self.players_iter().filter(|(x, y)| y.is_betting())
    }

    pub fn betting_players_iter_after(
        &self,
        i: usize,
    ) -> impl Iterator<Item = (usize, &SeatedPlayer)> + Clone + '_ {
        let si = if i >= MAX_PLAYERS - 1 { 0 } else { i };
        self.betting_players_iter()
            .cycle()
            .skip_while(move |(x, _)| x <= &si)
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
            player.bet_status = BetStatus::default();
        }
    }

    pub fn end_hand(&mut self) -> Result<(), GameError> {
        self.unfold_all();
        Ok(())
    }

    ///
    pub fn start_hand(&mut self) -> Result<Vec<i32>, GameError> {
        self.last_better = self.dealer_token;
        self.auto_fold_players();
        self.rotate_tokens()?;
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
        if self.betting_players_iter().count() < 3 {
            return Err(GameError::NotEnoughPlayers);
        }
        let od = self.dealer_token;
        let v = self
            .betting_players_iter_after(od)
            .map(|(x, _)| x)
            .collect::<Vec<usize>>();
        let mut iter = v.iter();
        self.dealer_token = *iter.next().ok_or(GameError::NotEnoughPlayers)?;
        // dealer_token can also be big blind
        self.small_blind_token = *iter.next().ok_or(GameError::NotEnoughPlayers)?;
        self.big_blind_token = *iter.next().ok_or(GameError::NotEnoughPlayers)?;
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
    pub fn bet(&mut self, bet: i32) -> Result<i32, BetError> {
        unimplemented!()
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
        self.monies >= 0
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

/// Handles all pot related operations.
/// Only tracks monies committed to the pot.
/// As such, does no error handling and cannot fail.
/// Parent must validate player has enough monies, and track the state of the betting round.
#[derive(Debug)]
pub struct Pot {
    players_in: HashMap<i32, i32>,
    max_in: i32,
    side_pot: Option<Box<Pot>>,
    is_settled: bool,
}

impl Pot {
    /// Returns the total value in this pot
    /// Not particularily useful due to each betting round spinning off a side pot
    pub fn value(&self) -> i32 {
        self.players_in.values().sum()
    }

    fn overflowing_add(&mut self, player: i32, amount: i32) {
        if self.is_settled {
            self.side_pot().overflowing_add(player, amount);
        } else {
            let ov = self.players_in.get(&player).copied().unwrap_or_default();
            let nv = ov + amount;
            if nv > self.max_in {
                self.players_in.insert(player, self.max_in);
                let o = nv - self.max_in;
                self.side_pot().overflowing_add(player, o);
            } else {
                self.players_in.insert(player, nv);
            }
        }
    }

    fn side_pot(&mut self) -> &mut Pot {
        if self.side_pot.is_some() {
            self.side_pot.as_mut().unwrap()
        } else {
            self.side_pot = Some(Box::new(Pot::default()));
            self.side_pot.as_mut().unwrap()
        }
    }

    fn update_max(&mut self, new_max: i32) {
        use std::cmp::Ordering;
        if self.is_settled {
            self.side_pot().update_max(new_max);
        } else {
            if new_max == i32::MAX || new_max < 1 {
                return;
            }
            match new_max.cmp(&self.max_in) {
                Ordering::Greater => self.side_pot().update_max(new_max),
                Ordering::Less => {
                    let ov = self.max_in;
                    self.max_in = new_max;
                    if ov != i32::MAX {
                        self.side_pot().update_max(ov - new_max);
                    }
                }
                Ordering::Equal => (),
            }
        }
    }

    /// Parent MUST call this in between betting rounds.
    /// Closes the betting round of all open pots. Next betting roung will create a fresh pot.
    /// This prevents confusion between max_in and next betting rounds.
    pub fn finalize_round(&mut self) {
        self.is_settled = true;
        if let Some(x) = self.side_pot.as_mut() {
            x.finalize_round();
        }
    }

    /// Detected a change in max_bet that could have consquences, forcing a rebuild
    fn overflow(&mut self) {
        if self.is_settled {
            self.side_pot().overflow();
        } else {
            for (player, value) in self.players_in.clone() {
                if value > self.max_in {
                    let delta = value - self.max_in;
                    self.players_in.insert(player, self.max_in);
                    self.overflowing_add(player, delta);
                }
            }
        }
    }

    /// Takes a vector of player Ids and returns the count of them that are in the current pot
    fn num_players_in(&self, hand: &[i32]) -> i32 {
        let mut r = 0;
        for i in hand {
            if self.players_in.contains_key(i) {
                r += 1;
            }
        }
        r
    }

    /// Consumes the pot and returns the total payout.
    ///
    /// # Panics
    ///
    /// Panics if the pot would pay out a different amount than is in the pot.
    /// This indicates a failure of the payout function and should be investigated.
    pub fn payout(self, ranked_hands: &[Vec<i32>]) -> HashMap<i32, i32> {
        let mut hm: HashMap<i32, i32> = HashMap::new();
        let value = self.value();
        let mut paid_out = false;
        for best_hand in ranked_hands {
            let hands_in = self.num_players_in(best_hand);
            // Prevents divide by zero below
            if hands_in == 0 {
                continue;
            }
            let payout = value / self.num_players_in(best_hand) as i32;
            for player in best_hand.iter() {
                if self.players_in.contains_key(player) {
                    hm.insert(*player, payout);
                    paid_out = true;
                    if best_hand.len() > 1 && value % 2 == 1 {
                        // TODO Randomize
                        hm.insert(best_hand[0], payout + 1);
                    }
                }
            }
            if paid_out {
                break;
            }
        }
        assert_eq!(hm.values().sum::<i32>(), self.value());
        if let Some(x) = self.side_pot {
            poker_core::util::merge_hashmap(&mut hm, x.payout(ranked_hands));
        }
        hm
    }

    /// Takes the players TOTAL bet. I.e. Bet(10), Call(20) = bet of 20.
    /// As such, parent must track the current betting round.
    pub fn bet(&mut self, player: i32, action: BetAction) -> i32 {
        use std::cmp::Ordering;
        if self.is_settled {
            self.side_pot().bet(player, action)
        } else {
            let ov = self.players_in.get(&player).copied().unwrap_or_default();
            let value = match action {
                BetAction::AllIn(v) => match v.cmp(&self.max_in) {
                    Ordering::Greater => {
                        let nv = v - self.max_in - ov;
                        self.players_in.insert(player, self.max_in);
                        self.side_pot().bet(player, BetAction::AllIn(nv))
                    }
                    Ordering::Equal => v,
                    Ordering::Less => {
                        self.update_max(v);
                        self.overflow();
                        v
                    }
                },
                BetAction::Bet(v) | BetAction::Call(v) => v,
                // Folds and calls have no effect on the pot.
                _ => return 0,
            };
            self.overflowing_add(player, value - ov);
            0
        }
    }
}

impl Default for Pot {
    fn default() -> Self {
        Pot {
            players_in: HashMap::new(),
            max_in: i32::MAX,
            side_pot: None,
            is_settled: false,
        }
    }
}

#[derive(Debug)]
pub enum BetError {
    HasNoMoney,
    BetLowerThanCall,
    InvalidCall,
    PlayerIsNotBetting,
    BadAction,
}

pub enum GameError {
    DeckError(deck::DeckError),
    BetError(BetError),
    NotEnoughPlayers,
}

impl From<deck::DeckError> for GameError {
    fn from(d: deck::DeckError) -> Self {
        GameError::DeckError(d)
    }
}

impl From<BetError> for GameError {
    fn from(d: BetError) -> Self {
        GameError::BetError(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pot() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Call(5));
        p.bet(3, BetAction::Call(5));
        let payout = p.payout(&vec![vec![1]]);
        assert_eq!(payout[&1], 15);
    }

    #[test]
    fn multi_winners() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Bet(5));
        p.bet(3, BetAction::Bet(5));
        let payout = p.payout(&vec![vec![1, 2]]);
        assert_eq!(payout[&1], 8);
        assert_eq!(payout[&2], 7);

        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Bet(5));
        p.bet(3, BetAction::Bet(6));
        let payout = p.payout(&vec![vec![1, 2]]);
        assert_eq!(payout[&1], 8);
        assert_eq!(payout[&2], 8);
    }

    #[test]
    fn over_bet() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Bet(5));
        p.bet(3, BetAction::Bet(6));
        let payout = p.payout(&vec![vec![1, 2], vec![3]]);
        assert_eq!(payout[&1], 8);
        assert_eq!(payout[&2], 7);
        assert_eq!(payout[&3], 1);
    }

    #[test]
    fn all_in_blind() {
        let mut p = Pot::default();
        p.bet(1, BetAction::AllIn(5));
        p.bet(2, BetAction::Bet(10));
        p.bet(3, BetAction::AllIn(8));
        dbg!(&p);
        let payout = p.payout(&vec![vec![1], vec![2, 3]]);
        dbg!(&payout);
        assert_eq!(payout[&1], 15);
        assert_eq!(payout[&2], 5);
        assert_eq!(payout[&3], 3);
    }

    #[test]
    fn side_pot_payout() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(10));
        p.bet(2, BetAction::AllIn(5));
        p.bet(3, BetAction::Bet(10));
        let payout = p.payout(&vec![vec![2], vec![1, 3]]);
        assert_eq!(payout[&2], 15);
        assert_eq!(payout[&1], 5);
        assert_eq!(payout[&3], 5);
    }

    #[test]
    fn overflowing_side_pot() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(10));
        p.bet(2, BetAction::AllIn(5));
        p.bet(3, BetAction::AllIn(3));
        dbg!(&p);
        let payout = p.payout(&vec![vec![3], vec![2], vec![1]]);
        dbg!(&payout);
        assert_eq!(payout[&3], 9);
        assert_eq!(payout[&2], 4);
        // 1 overbet and was returned pot nobody else could claim
        assert_eq!(payout[&1], 5);
    }

    #[test]
    fn multi_round_pot() {
        let mut p = Pot::default();
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Call(5));
        p.bet(3, BetAction::Call(5));
        p.finalize_round();
        // 5,5,5 = 15 in pot
        p.bet(1, BetAction::Bet(5));
        p.bet(2, BetAction::Bet(10));
        p.bet(3, BetAction::AllIn(8));
        p.bet(1, BetAction::Call(10));
        p.finalize_round();
        // 15 + 8,8,8 + 2,2 = 43 in pot
        p.bet(1, BetAction::Bet(10));
        p.bet(2, BetAction::AllIn(6));
        // 43 + 6,6 + 4 = 59 in pot
        dbg!(&p);
        let payout = p.payout(&vec![vec![3], vec![2], vec![1]]);
        dbg!(&payout);
        assert_eq!(payout[&3], 39);
        assert_eq!(payout[&2], 16);
        // 1 overbet and was returned pot nobody else could claim
        assert_eq!(payout[&1], 4);
    }
}
