use super::deck::{Card, Deck};
use super::players::SeatedPlayers;
use super::pot::Pot;

use super::{BetAction, GameError};
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
    Dealing,
    Betting(i32, BetRound),
    // This isn't right
    Winner(i32, i32),
    // This isn't right
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
    pub current_bet: i32,
    d: Deck,
}

impl GameInProgress {
    pub fn start_round(&mut self) -> Result<(), GameError> {
        self.state = GameState::Dealing;
        self.d = Deck::default();
        // TODO save seed

        // Handles auto folds and moving the tokens
        let _seated_players = self.seated_players.start_hand()?;

        // Reset the pot
        self.pots = Pot::default();

        // Blinds bet
        let first_better = self
            .seated_players
            .blinds_bet(self.small_blind, self.big_blind())?;

        self.state = GameState::Betting(first_better, BetRound::PreFlop(self.big_blind()));

        // Deal the pockets
        let np = self.seated_players.betting_players_iter().count() as u8;
        let _pockets = self.d.deal_pockets(np)?;

        Ok(())
    }

    pub fn bet(&mut self, _player: i32, ba: BetAction) -> Result<i32, GameError> {
        // Convert check into related call
        let _ba = if matches!(ba, BetAction::Check) {
            BetAction::Call(self.current_bet)
        } else {
            ba
        };
        // Make sure calls equal the current bet
        // Make sure bets are >= current bet
        // Call seated player bet
        // Update Pot
        // Play pending action for next better
        // Determine if this was the final bet and round is over
        unimplemented!()
    }

    /// Simple abstraction so can make big blinds that are not x2 later
    fn big_blind(&self) -> i32 {
        self.small_blind * 2
    }

    fn _finalize_hand(&mut self) -> Result<GameState, GameError> {
        self.seated_players.end_hand()?;
        // TODO Fold 'auto-fold' players?
        // TODO Force rocket to update DB? Probably by returning State enum?
        unimplemented!()
    }
}
