use crate::game::BetError;

use super::deck::{Card, Deck};
use super::players::{PlayerId, SeatedPlayer, SeatedPlayers};
use super::pot::Pot;

use super::{BetAction, Currency, GameError};
use derive_more::Display;
use std::cmp::Ordering;

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

#[derive(Debug)]
pub enum GameState {
    NotStarted,
    Dealing,
    Betting(BetRound),
    // This isn't right
    Winner(i32, i32),
    // This isn't right
    WinnerDuringBet(i32, i32),
}

#[derive(Debug, Copy, Clone)]
pub enum BetRound {
    PreFlop,
    Flop,
    Turn,
    River,
}

#[derive(Debug, PartialEq, Clone)]
pub enum GameEvent {
    NewDeckSeed(String),
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
            hand_num: 0,
            event_log: Vec::new(),
            deck: Deck::default(),
        }
    }
}

#[derive(Debug)]
pub struct GameInProgress {
    table_type: TableType,
    pub table_cards: [Option<Card>; 5],
    seated_players: SeatedPlayers,
    pub pot: Pot,
    pub state: GameState,
    pub small_blind: Currency,
    pub current_bet: Currency,
    pub min_raise: Currency,
    pub hand_num: i16,
    pub event_log: Vec<GameEvent>,
    deck: Deck,
}

impl GameInProgress {
    pub fn start_round(&mut self) -> Result<(), GameError> {
        self.state = GameState::Dealing;
        let (deck, _seed) = Deck::deck_and_seed();
        self.deck = deck;

        // TODO save seed for DB, and perhaps the log
        self.hand_num += 1;

        // Handles auto folds and moving the okens
        self.seated_players.start_hand()?;

        // TODO log players in hand

        // Reset the pot
        self.pot = Pot::default();

        // Blinds bet
        let (small_blind, big_blind) = self
            .seated_players
            .blinds_bet(self.small_blind, self.big_blind())?;

        self.pot.bet(small_blind.0, small_blind.1);
        self.pot.bet(big_blind.0, big_blind.1);
        // TODO Log Blinds, perhaps edit Pot::bet (and its other funcs?) to return pot LogItems

        self.state = GameState::Betting(BetRound::PreFlop);

        // Deal the pockets
        let nump = self.seated_players.betting_players_count() as u8;
        let pockets = self.deck.deal_pockets(nump)?;
        //println!("{:?}", pockets);
        self.seated_players.deal_pockets(pockets);

        Ok(())
    }

    /// Gets the seated player by id if they are seated at the current table.
    /// Front-end is responsible for making sure there isn't data leakage
    pub fn get_player_info(&self, player_id: PlayerId) -> Option<&SeatedPlayer> {
        self.seated_players.player_by_id(player_id).map(|x| &*x)
    }

    pub fn sit_down<C: Into<Currency>>(
        &mut self,
        player_id: PlayerId,
        monies: C,
        seat: usize,
    ) -> Result<(), GameError> {
        self.seated_players.sit_down(player_id, monies, seat)
    }

    pub fn stand_up(&mut self, player_id: PlayerId) -> Option<Result<Currency, GameError>> {
        match self.state {
            GameState::Winner(..) | GameState::WinnerDuringBet(..) => {
                self.seated_players.stand_up(player_id).map(Ok)
            }
            _ => {
                let p = self.seated_players.player_by_id(player_id)?;
                if p.is_betting() {
                    Some(Err(GameError::BettingPlayerCantStand))
                } else {
                    self.seated_players.stand_up(player_id).map(Ok)
                }
            }
        }
    }

    pub fn bet(&mut self, player: PlayerId, ba: BetAction) -> Result<Currency, GameError> {
        // Make sure we're in a state where bets are expected
        match self.state {
            GameState::Betting(_) => {}
            _ => return Err(GameError::BetNotExpected),
        }
        // Make sure bet is for an appropriate amount
        match &ba {
            BetAction::Bet(x) | BetAction::Call(x) => {
                match x.cmp(&self.current_bet) {
                    Ordering::Less => return Err(BetError::BetTooLow.into()),
                    Ordering::Greater => return Err(BetError::BetTooHigh.into()),
                    // No errors to account for and no maintenance to do
                    Ordering::Equal => {}
                }
            }
            // No errors to account for and no maintenance to do
            BetAction::Check | BetAction::Fold => {}
            // AllIn can be for any amount, so no errors to catch
            BetAction::AllIn(_) => {}
            BetAction::Raise(x) => {
                // A raise is only in error if it doesn't meet the min raise
                if x < &self.min_raise {
                    return Err(BetError::BetTooLow.into());
                }
            }
        }

        // Call seated players bet, which will convert to AllIn as neccesary
        let new_ba_and_current_bet = self.seated_players.bet(player, ba, self.current_bet)?;
        let new_ba = new_ba_and_current_bet.0;
        let old_current_bet = self.current_bet;
        self.current_bet = new_ba_and_current_bet.1;
        if old_current_bet != self.current_bet {
            self.min_raise = old_current_bet + (self.current_bet - old_current_bet);
        }
        // Update Pot
        self.pot.bet(player, new_ba);

        // Determine if this was the final bet and round is over
        if self.seated_players.is_pot_ready(self.current_bet) {
            // Advance game state
            let new_round = match self.state {
                GameState::Betting(round) => match round {
                    BetRound::PreFlop => {
                        self.deck.burn();
                        self.table_cards[0] = Some(self.deck.draw()?);
                        self.table_cards[1] = Some(self.deck.draw()?);
                        self.table_cards[2] = Some(self.deck.draw()?);
                        // reset bet status for all betting players
                        BetRound::Flop
                    }
                    BetRound::Flop => {
                        self.deck.burn();
                        self.table_cards[3] = Some(self.deck.draw()?);
                        // reset bet status for all betting players
                        BetRound::Turn
                    }
                    BetRound::Turn => {
                        self.deck.burn();
                        self.table_cards[4] = Some(self.deck.draw()?);
                        // reset bet status for all betting players
                        BetRound::River
                    }
                    BetRound::River => todo!(),
                },
                _ => unreachable!(),
            };
            self.state = GameState::Betting(new_round);
            // Inform seated_players of new round
        }
        Ok(0.into())
    }

    /// Simple abstraction so can make big blinds that are not x2 later
    fn big_blind(&self) -> Currency {
        self.small_blind * 2
    }

    fn _finalize_hand(&mut self) -> Result<GameState, GameError> {
        self.seated_players.end_hand()?;
        // TODO 'stand_up' players who are trying to leave but couldn't because they were in the bet?
        // TODO Force rocket to update DB? Probably by returning State enum?
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_game() {
        let mut gt = GameInProgress::default();
        gt.sit_down(0, 100, 0).unwrap();
        gt.sit_down(1, 100, 1).unwrap(); // small blind
        gt.sit_down(2, 100, 2).unwrap(); // big blind
        gt.sit_down(3, 100, 3).unwrap();
        gt.start_round().unwrap();
        // Blinds are in
        assert_eq!(gt.get_player_info(0).unwrap().monies(), 100.into());
        assert_eq!(gt.get_player_info(1).unwrap().monies(), 95.into());
        assert_eq!(gt.get_player_info(2).unwrap().monies(), 90.into());
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
        assert_eq!(gt.get_player_info(0).unwrap().monies(), 100.into());
        assert_eq!(gt.get_player_info(1).unwrap().monies(), 90.into());
        assert_eq!(gt.get_player_info(2).unwrap().monies(), 90.into());
        assert_eq!(gt.get_player_info(3).unwrap().monies(), 90.into());
        assert_eq!(gt.pot.total_value(), 30.into());
        assert!(gt.table_cards[0].is_some());
        assert!(gt.table_cards[1].is_some());
        assert!(gt.table_cards[2].is_some());
        assert!(gt.table_cards[3].is_none());
        assert!(gt.table_cards[4].is_none());

        // TODO rest of the test once the above passes
    }

    // TODO test where players who are folded try to bet again

    // TODO test where players that are currently bet eligible try to stand up

    // TODO moar
}
