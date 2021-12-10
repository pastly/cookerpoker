use super::deck::{Card, Deck};
use super::players::{PlayerId, SeatedPlayer, SeatedPlayers};
use super::pot::Pot;

use super::{BetAction, Currency, GameError};
use derive_more::Display;

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
    Betting(PlayerId, BetRound),
    // This isn't right
    Winner(i32, i32),
    // This isn't right
    WinnerDuringBet(i32, i32),
}

#[derive(Debug)]
pub enum BetRound {
    PreFlop(Currency),
    Flop(Currency),
    Turn(Currency),
    River(Currency),
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
            small_blind: 10.into(),
            current_bet: 10.into(),
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
    pub seated_players: SeatedPlayers,
    pub pot: Pot,
    pub state: GameState,
    pub small_blind: Currency,
    pub current_bet: Currency,
    pub hand_num: i16,
    pub event_log: Vec<GameEvent>,
    deck: Deck,
}

impl GameInProgress {
    pub fn start_round(&mut self) -> Result<(), GameError> {
        self.state = GameState::Dealing;
        let (deck, _seed) = Deck::deck_and_seed();
        self.deck = deck;

        // TODO save seed for DB
        self.hand_num += 1;

        // Handles auto folds and moving the tokens
        let _players_in = self.seated_players.start_hand()?;

        // TODO log players in hand

        // Reset the pot
        self.pot = Pot::default();

        // Blinds bet
        let (small_blind, big_blind, first_better) = self
            .seated_players
            .blinds_bet(self.small_blind, self.big_blind())?;

        self.pot.bet(small_blind.0, small_blind.1);
        self.pot.bet(big_blind.0, big_blind.1);
        // TODO Log Blinds

        self.state = GameState::Betting(first_better, BetRound::PreFlop(self.big_blind()));

        // Deal the pockets
        let nump = self.seated_players.betting_players_iter().count() as u8;
        let pockets = self.deck.deal_pockets(nump)?;
        self.seated_players.deal_pockets(pockets);

        Ok(())
    }

    /// Gets the seated player by id if they are seated at the current table.
    /// Front-end is responsible for making sure there isn't data leakage
    pub fn get_player_info(&self, player_id: PlayerId) -> Option<&SeatedPlayer> {
        self.seated_players.player_by_id(player_id).map(|x| &*x)
    }

    pub fn sit_down(
        &mut self,
        player_id: PlayerId,
        monies: Currency,
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
        // Convert check into related call
        // TODO OR return an error and not accept the check?
        let ba = if matches!(ba, BetAction::Check) {
            BetAction::Call(self.current_bet)
        } else {
            ba
        };
        // Make sure calls equal the current bet
        // Make sure bets are >= current bet
        // Call seated players bet, which will convert to AllIn as neccesary
        // Update Pot
        self.pot.bet(player, ba);
        // Play pending action for next better
        // Determine if this was the final bet and round is over
        unimplemented!()
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
        gt.sit_down(0.into(), 100.into(), 0).unwrap();
        gt.sit_down(1.into(), 100.into(), 1).unwrap();
        gt.sit_down(2.into(), 100.into(), 2).unwrap();
        gt.sit_down(3.into(), 100.into(), 3).unwrap();
        gt.start_round().unwrap();
        // Blinds are in
        assert_eq!(gt.get_player_info(0.into()).unwrap().monies(), 100.into());
        assert_eq!(gt.get_player_info(1.into()).unwrap().monies(), 95.into());
        assert_eq!(gt.get_player_info(2.into()).unwrap().monies(), 90.into());
        assert_eq!(gt.pot.total_value(), 15.into());

        gt.bet(3.into(), BetAction::Call(10.into())).unwrap();
        gt.bet(0.into(), BetAction::Fold).unwrap();
        // TODO decide if invald Check's should fail or be converted to calls
        let _r = gt.bet(1.into(), BetAction::Check).unwrap();
        gt.bet(2.into(), BetAction::Check).unwrap();

        // First betting round is over.
        // Table should recognize that all players are in and pot is right and forward the round
        assert_eq!(gt.get_player_info(0.into()).unwrap().monies(), 100.into());
        assert_eq!(gt.get_player_info(1.into()).unwrap().monies(), 90.into());
        assert_eq!(gt.get_player_info(2.into()).unwrap().monies(), 90.into());
        assert_eq!(gt.get_player_info(3.into()).unwrap().monies(), 90.into());
        assert_eq!(gt.pot.total_value(), 30.into());
        assert!(gt.table_cards[2].is_some());

        // TODO rest of the test once the above passes
    }

    // TODO test where players who are folded try to bet again

    // TODO test where players that are currently bet eligible try to stand up

    // TODO moar
}
