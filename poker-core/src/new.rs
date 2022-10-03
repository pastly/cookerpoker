use serde::{Deserialize, Serialize};

// pub? Old one was, but why needed?
const MAX_PLAYERS: usize = 12;
pub type PlayerId = i32;
pub type Currency = i32;

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum GameError {
    PlayerAlreadySeated,
    TableFull,
}

/// (Replaces TableType)
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum TableType {
    Cash,
}

impl Default for TableType {
    fn default() -> Self {
        Self::Cash
    }
}

/// GameState, but filtered to just the state that a given player should be able to see. I.e. while
/// GameState needs to know all hole cards, this will only reveal the hole cards of a single player
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct FilteredGameState {
    table_type: TableType,
    pub players: Players,
}

impl FilteredGameState {
    pub fn is_cash(&self) -> bool {
        matches!(self.table_type, TableType::Cash)
    }
}

/// (Replaces GameInProgress) All the state constituting a poker game in progress
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    table_type: TableType,
    players: Players,
}

impl GameState {
    pub fn filter(&self, _player_id: PlayerId) -> FilteredGameState {
        FilteredGameState {
            table_type: self.table_type,
            players: self.players,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            table_type: Default::default(),
            players: Default::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Players {
    players: [Option<Player>; MAX_PLAYERS],
}

impl Default for Players {
    fn default() -> Self {
        Self {
            players: [None; MAX_PLAYERS],
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub stack: Currency,
}

impl GameState {
    pub fn try_sit(&mut self, player_id: PlayerId, stack: Currency) -> Result<(), GameError> {
        if self.players.player_by_id(player_id).is_some() {
            return Err(GameError::PlayerAlreadySeated);
        }
        let p = Player::new(player_id, stack);
        self.players.seat_player(p)?;
        Ok(())
    }
    
    /// DEV-only: reset game state to a clean stating state
    /// 
    /// Remove all cards from everywhere.
    /// Move button to somewhere new.
    /// Basic clean up stuff like that.
    /// 
    /// Leave players' seat positions and their stacks alone
    pub fn devonly_reset(&mut self) {
        ()
    }
}

impl Players {
    fn player_by_id(&self, id: PlayerId) -> Option<&Player> {
        self.players_iter().find(|x| x.id == id)
    }

    fn seat_player(&mut self, player: Player) -> Result<usize, GameError> {
        if let Some(seat_idx) = self.next_empty_seat() {
            self.players[seat_idx] = Some(player);
            Ok(seat_idx)
        } else {
            Err(GameError::TableFull)
        }
    }

    fn next_empty_seat(&self) -> Option<usize> {
        match self
            .players
            .iter()
            .enumerate()
            .find(|(_idx, p)| p.is_none())
        {
            Some((idx, _p)) => Some(idx),
            None => None,
        }
    }

    pub fn players_iter(&self) -> impl Iterator<Item = &Player> /*+ Clone + '_ */ {
        self.players
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.as_ref().unwrap())
    }
}

impl Player {
    fn new(id: PlayerId, stack: Currency) -> Self {
        Self { id, stack }
    }
}
