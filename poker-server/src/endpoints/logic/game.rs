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
pub enum PotAction {
    Check,
    Fold,
    Call(i32),
    Bet(i32),
    AllIn(i32),
}

pub enum BetStatus {
    Folded,
    Waiting,
    In(i32),
    AllIn(i32),
}

pub enum GameState {
    Dealing,
    Betting(i32),
    Winner(i32),
    WinnerDuringBet(i32),
}

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
    players: [SeatedPlayer; MAX_PLAYERS],
    last_better: usize,
    dealer: usize,
    small_blind: usize,
    big_blind: usize,
}

impl SeatedPlayers {
    /// Moves betting round forward and returns account id of next better
    /// Returns None if betting round is over.
    pub fn next_better(&mut self) -> Option<i32> {
        // Don't want to ask last better to bet again
        self.last_better += 1;
        let np = self.player_after(self.last_better).ok()?;
        self.players.get(np).map(|x| x.id)
    }

    /// Returns the next active player after the supplied index
    /// Player must not have folded and have more than 0 monies
    fn player_after(&self, i: usize) -> Result<usize, GameError> {
        //Brute force until I figure out a better way
        for np in i..MAX_PLAYERS {
            if self.players[i].has_monies() && !self.players[i].is_folded {
                return Ok(np);
            }
        }
        for np in 0..i {
            if self.players[i].has_monies() && !self.players[i].is_folded {
                return Ok(np);
            }
        }
        Err(GameError::NotEnoughPlayers)
    }

    /// Returns the number of players still in.
    /// Returning a value of 1 means the round is over
    /// Should never return 0, as that would indicate the hand winner folded
    pub fn num_players_still_in(&self) -> u8 {
        let r = self.players.len() as u8
            - 1u8
            - self
                .players
                .iter()
                .fold(0u8, |c, s| if s.is_folded { c + 1 } else { c });
        assert_ne!(r, 0);
        r
    }

    fn unfold_all(&mut self) {
        for player in self.players.iter_mut() {
            player.is_folded = false;
        }
    }

    /// Returns a vector of account ids for players that are still active
    pub fn get_active_players(&self) -> Vec<i32> {
        let mut v = Vec::new();
        for player in self.players.iter() {
            if player.has_monies() && !player.is_folded {
                v.push(player.id);
            }
        }
        v
    }

    pub fn end_hand(&mut self) -> Result<(), GameError> {
        self.unfold_all();
        Ok(())
    }

    ///
    pub fn start_hand(&mut self) -> Result<Vec<i32>, GameError> {
        self.last_better = self.dealer;
        self.fold_broke_players();
        self.rotate_tokens()?;
        Ok(self.get_active_players())
    }

    fn fold_broke_players(&mut self) {
        for player in self.players.iter_mut() {
            if !player.has_monies() {
                player.is_folded = true;
            }
        }
    }

    fn rotate_tokens(&mut self) -> Result<(), GameError> {
        self.dealer = self.player_after(self.dealer)?;
        self.small_blind = self.player_after(self.dealer)?;
        // Dealer can also be big blind
        self.big_blind = self.player_after(self.small_blind).unwrap_or(self.dealer);
        Ok(())
    }
}

#[derive(Debug)]
pub struct SeatedPlayer {
    pub id: i32,
    pub pocket: Option<[Card; 2]>,
    monies: i32,
    pub is_folded: bool,
}

impl SeatedPlayer {
    pub fn bet(&mut self, bet: i32) -> Result<i32, PotError> {
        unimplemented!()
    }
    /*if self.monies == 0 {
            return Err(PotError::HasNoMoney);
        }
        let d = self.monies - bet;
        if d.is_positive() {
            self.monies -= bet;
            Ok(bet)
        } else {
            // Does not have enogh to match bet. All in.
            self.monies = 0;
            Err(Action::AllIn(self.monies))
        }
    }*/
    pub fn has_monies(&self) -> bool {
        self.monies >= 0
    }
}

#[derive(Debug)]
pub struct Pot {
    players_in: HashMap<i32, i32>,
    pub current_bet: i32,
    max_in: i32,
    side_pot: Option<Box<Pot>>,
}

impl Pot {
    pub fn new(current_bet: i32) -> Self {
        Pot {
            players_in: HashMap::new(),
            max_in: i32::MAX,
            side_pot: None,
            current_bet,
        }
    }

    pub fn value(&self) -> i32 {
        self.players_in.values().sum()
    }

    fn overflowing_add(&mut self, player: i32, amount: i32) {
        let ov = self.players_in.get(&player).copied().unwrap_or_default();
        let nv = ov + amount;
        if nv > self.max_in {
            self.players_in.insert(player, self.max_in);
            let o = nv - self.max_in;
            self.side_pot(self.current_bet).overflowing_add(player, o);
        } else {
            self.players_in.insert(player, nv);
        }
    }

    fn side_pot(&mut self, current_bet: i32) -> &mut Pot {
        if self.side_pot.is_some() {
            self.side_pot.as_mut().unwrap()
        } else {
            self.side_pot = Some(Box::new(Pot::new(current_bet)));
            self.side_pot.as_mut().unwrap()
        }
    }

<<<<<<< HEAD
    fn update_max(&mut self, new_max: i32) {
        if new_max == i32::MAX {
            return
        }
        let ov = self.max_in;
        self.side_pot(self.current_bet).update_max(ov);
        self.max_in = new_max;
    }

=======
>>>>>>> 657a9304df2b16ce7fc682b49face86454da7e21
    /// Detected a change in max_bet that could have consquences, forcing a rebuild
    fn overflow(&mut self) -> Result<i32, PotError> {
        let mut needs_reset = false;
        for (player, value) in self.players_in.clone() {
            if value > self.max_in {
                let delta = value - self.max_in;
                self.players_in.insert(player, self.max_in);
                self.overflowing_add(player, delta);
                needs_reset = true;
            }
        }
        if needs_reset {

        }
        Ok(0)
    }

    pub fn payout(self, ranked_hands: &Vec<Vec<i32>>) -> HashMap<i32, i32> {
        let mut hm: HashMap<i32, i32> = HashMap::new();
        let value = self.value();
        let mut paid_out = false;
        for best_hand in ranked_hands {
            let payout = value / best_hand.len() as i32;
            for player in best_hand.iter() {
                if self.players_in.contains_key(player) {
                    hm.insert(*player, payout);
                    paid_out = true;
                }
                if best_hand.len() > 1 && value % 2 == 1 {
                    // TODO Randomize
                    hm.insert(best_hand[0], payout + 1);
                }
            }
            if paid_out {
                break;
            }
        }
        assert_eq!(hm.values().sum::<i32>(), self.value());
        match self.side_pot {
            Some(x) => poker_core::util::merge_hashmap(&mut hm, x.payout(ranked_hands)),
            None => (),
        }
        hm
    }

    pub fn bet(&mut self, player: i32, action: PotAction) -> Result<i32, PotError> {
        let ov = self.players_in.get(&player).copied().unwrap_or_default();
        let value = match action {
            PotAction::AllIn(v) => {
                let nv = ov + v;
                let below = nv - 1;
                let above = nv + 1;
                if self.max_in < nv {
                    match &mut self.side_pot {
                        Some(x) => x.bet(player, PotAction::AllIn(v)),
<<<<<<< HEAD
                        None => {self.update_max(nv);self.overflow() },
                    }
                } else {
                    self.update_max(v);
=======
                        None => self.overflow(),
                    }
                } else {
                    self.max_in = v;
>>>>>>> 657a9304df2b16ce7fc682b49face86454da7e21
                    self.overflow()?;
                    Ok(v)
                }
            }
            PotAction::Bet(v) => {
                if v < self.current_bet {
                    Err(PotError::BetLowerThanCall)
                } else {
                    Ok(v)
                }
            }
            PotAction::Call(v) => {
                if v != self.current_bet {
                    Err(PotError::InvalidCall)
                } else {
                    Ok(v)
                }
            }
            _ => Err(PotError::BadAction),
        }?;
        self.overflowing_add(player, value);
        Ok(0)
    }
}

impl Default for Pot {
    fn default() -> Self {
                    Pot {
                players_in: HashMap::new(),
                max_in: i32::MAX,
                side_pot: None,
                current_bet: 0,
            }
        
    }
}

#[derive(Debug)]
pub enum PotError {
    HasNoMoney,
    BetLowerThanCall,
    InvalidCall,
    BadAction,
}

pub enum GameError {
    DeckError(deck::DeckError),
    NotEnoughPlayers,
}

impl From<deck::DeckError> for GameError {
    fn from(d: deck::DeckError) -> Self {
        GameError::DeckError(d)
    }
}

#[cfg(test)]
mod tests {
    use rocket::figment::error::Actual;

    use super::*;

    #[test]
    fn basic_pot() {
        let mut p = Pot::new(5);
        p.bet(1, PotAction::Bet(5)).unwrap();
        p.bet(2, PotAction::Call(5)).unwrap();
        p.bet(3, PotAction::Call(5)).unwrap();
        let payout = p.payout(&vec![vec![1]]);
        assert_eq!(payout[&1], 15);
    }

    #[test]
    fn multi_winners() {
        let mut p = Pot::new(5);
        p.bet(1, PotAction::Bet(5)).unwrap();
        p.bet(2, PotAction::Bet(5)).unwrap();
        p.bet(3, PotAction::Bet(5)).unwrap();
        let payout = p.payout(&vec![vec![1, 2]]);
        assert_eq!(payout[&1], 8);
        assert_eq!(payout[&2], 7);

        let mut p = Pot::new(5);
        p.bet(1, PotAction::Bet(5)).unwrap();
        p.bet(2, PotAction::Bet(5)).unwrap();
        p.bet(3, PotAction::Bet(6)).unwrap();
        let payout = p.payout(&vec![vec![1, 2]]);
        assert_eq!(payout[&1], 8);
        assert_eq!(payout[&2], 8);
    }

    #[test]
    fn all_in_blind() {
        let mut p = Pot::new(5);
        p.bet(1, PotAction::AllIn(5)).unwrap();
        p.bet(2, PotAction::Bet(10)).unwrap();
        p.bet(3, PotAction::AllIn(8)).unwrap();
        dbg!(&p);
        let payout = p.payout(&vec![vec![1], vec![2, 3]]);
        dbg!(&payout);
        assert_eq!(payout[&1], 15);
        assert_eq!(payout[&2], 2);
        assert_eq!(payout[&3], 6);
    }

    #[test]
    fn side_pot_payout() {
        let mut p = Pot::new(5);
        p.bet(1, PotAction::Bet(10)).unwrap();
        p.bet(2, PotAction::AllIn(5)).unwrap();
        p.bet(3, PotAction::Bet(10)).unwrap();
        let payout = p.payout(&vec![vec![2], vec![1, 3]]);
        assert_eq!(payout[&2], 15);
        assert_eq!(payout[&1], 5);
        assert_eq!(payout[&3], 5);
    }

    #[test]
    fn overflowing_side_pot() {
        let mut p = Pot::new(5);
        p.bet(1, PotAction::Bet(10)).unwrap();
        p.bet(2, PotAction::AllIn(5)).unwrap();
        p.bet(3, PotAction::AllIn(3)).unwrap();
        p.bet(3, PotAction::AllIn(3)).unwrap();
        dbg!(&p);
        let payout = p.payout(&vec![vec![3], vec![2], vec![1]]);
        dbg!(&payout);
        assert_eq!(payout[&3], 9);
        assert_eq!(payout[&2], 4);
        // 1 overbet and was returned pot nobody else could claim
        assert_eq!(payout[&1], 5);
    }
}
