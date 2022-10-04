use crate::bet::BetAction;
use crate::deck::{Card, Deck};
use crate::player::{Player, Players};
use crate::pot::Pot;
use crate::GameError;
use crate::{Currency, PlayerId};
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

const COMMUNITY_SIZE: usize = 5;
const DEF_SB: Currency = 5;
const DEF_BB: Currency = 10;

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
    //state: State,
    table_type: TableType,
    pub players: Players,
    /*pub*/ community: [Option<Card>; COMMUNITY_SIZE],
}

impl FilteredGameState {
    pub fn is_cash(&self) -> bool {
        matches!(self.table_type, TableType::Cash)
    }
}

/// States a game can be in, e.g. not even stardard, dealing, showdown, etc.
#[derive(Debug, PartialEq, Eq, Clone, Copy, derive_more::Display, Serialize, Deserialize)]
pub enum State {
    NotStarded,
    Dealing,
    Street(Street),
    Showdown,
    EndOfHand,
}

impl Default for State {
    fn default() -> Self {
        Self::NotStarded
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
    state: State,
    /// Cash. Maybe tourny in the future
    table_type: TableType,
    /// The players seated at this table and their per-player info
    players: Players,
    /// The community cards
    community: [Option<Card>; COMMUNITY_SIZE],
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
    pub current_bet: Currency,
    /// If a player wishes to raise this betting round, they must raise to at least this amount.
    /// This is the total amount to raise to, i.e. it is larger than current_bet.
    pub min_raise: Currency,
    /// The last person to raise this betting round.
    ///
    /// Needed because of the full bet rule. You can't raise, have action come back to you, then
    /// raise again without someone raising after your first raise. Action can come back to you
    /// like this if someone goes all in for less than the minimum raise after your first raise.
    ///
    /// It's confusing. See <https://duckduckgo.com/?t=ffab&q=allin+raise+less+than+minraise>
    last_raiser: Option<PlayerId>,
}

impl GameState {
    pub fn filter(&self, _player_id: PlayerId) -> FilteredGameState {
        FilteredGameState {
            table_type: self.table_type,
            players: self.players.clone(),
            community: self.community,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            state: Default::default(),
            table_type: Default::default(),
            players: Default::default(),
            community: [None; COMMUNITY_SIZE],
            pot: Default::default(),
            deck: Default::default(),
            small_blind: DEF_SB,
            big_blind: DEF_BB,
            current_bet: DEF_BB,
            min_raise: 2 * DEF_BB,
            last_raiser: None,
        }
    }
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

    pub fn start_hand(&mut self) -> Result<(), GameError> {
        if self.players.players_iter().count() < 2 {
            return Err(GameError::NotEnoughPlayers);
        }
        self.state = State::Dealing;
        self.deck = Default::default();
        self.community = [None; COMMUNITY_SIZE];
        self.pot = Default::default();
        self.last_raiser = None;
        self.players.start_hand()?;

        self.state = State::Street(Street::PreFlop);
        self.current_bet = 0;
        let ((player_sb, bet_sb), (player_bb, bet_bb)) = self.blinds_bet()?;
        self.current_bet = self.big_blind;
        self.pot.bet(player_sb, bet_sb);
        self.pot.bet(player_bb, bet_bb);

        let num_p = self.players.betting_players_count() as u8;
        let pockets = self.deck.deal_pockets(num_p)?;
        self.players.deal_pockets(pockets);
        self.current_bet = self.big_blind;
        Ok(())
    }

    /// Have the SB and BB execute their obligatory preflop betting. Return their IDs and bet
    /// amounts.
    ///
    /// Caller can't assume SB and BB are in for the full SB/BB amount: they could have been a very
    /// short stack and now be allin for less.
    fn blinds_bet(&mut self) -> Result<((PlayerId, BetAction), (PlayerId, BetAction)), GameError> {
        let player_sb =
            self.players.players[self.players.token_sb].ok_or(GameError::PlayerNotFound)?;
        let player_bb =
            self.players.players[self.players.token_bb].ok_or(GameError::PlayerNotFound)?;
        let bet_sb = BetAction::Bet(self.small_blind);
        let bet_bb = BetAction::Bet(self.big_blind);
        self.bet(player_sb.id, bet_sb)?;
        self.bet(player_bb.id, bet_bb)?;
        // the blinds have bet, and we need to make sure they have the opportunity to bet again this
        // round, so rebuild need_bets_from
        self.players.need_bets_from = self
            .players
            .betting_players_iter_after(self.players.token_bb)
            .map(|(i, _)| i)
            .take(self.players.betting_players_count())
            .collect();
        self.players.need_bets_from.reverse();
        Ok(((player_sb.id, bet_sb), (player_bb.id, bet_bb)))
    }

    /// Check that the player can make the given bet, adjusting it if possible. Returns the
    /// (possibly adjusted) bet this player made
    fn bet(&mut self, player_id: PlayerId, bet: BetAction) -> Result<(), GameError> {
        // Check for obvious errors: game not in correct state
        if !matches!(self.state, State::Street(_)) {
            return Err(GameError::NoBetExpected);
        }
        // Check for obvious errors: bet too small, or this player shouldn't be betting, etc.
        match &bet {
            // nothing obvious to check for
            BetAction::Check | BetAction::Fold => {}
            // can be for any amount, so no errors to catch
            BetAction::AllIn(_) => {}
            BetAction::Bet(x) | BetAction::Call(x) => {
                match x.cmp(&self.current_bet) {
                    Ordering::Less => return Err(GameError::InvalidBet),
                    Ordering::Greater => {
                        // only an error if there is a non-zero current bet. It's 0 for the start of
                        // post-flop rounds
                        if self.current_bet != 0 {
                            return Err(GameError::InvalidBet);
                        }
                    }
                    // No errors to account for and no maintenance to do
                    Ordering::Equal => {}
                }
            }
            BetAction::Raise(x) => {
                if x < &self.min_raise {
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
                *x >= self.min_raise
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
                match x.cmp(&self.current_bet) {
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
                        let n = if bet.is_allin() && self.players.betting_players_count() == 0 {
                            0
                        } else if bet.is_allin() {
                            self.players.betting_players_count()
                        } else {
                            self.players.betting_players_count() - 1
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
        Ok(())
    }

    /// DEV-only: reset game state to a clean stating state. This func should not panic!
    ///
    /// Remove all cards from everywhere.
    /// Move button to somewhere new?
    /// Basic clean up stuff like that.
    ///
    /// Leave players' seat positions and their stacks alone
    pub fn devonly_reset(&mut self) {
        self.players.devonly_reset();
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
            BetStatus::In(10)
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
                while gs.players.token_dealer != first as usize {
                    gs.players.start_hand().unwrap();
                }
                let mut deck = Deck::default();
                let pockets = deck.deal_pockets(n_players as u8).unwrap();
                // this is the actual test. Does this panic?
                gs.players.deal_pockets(pockets);
                // okay so it didn't. let's make sure every player has a pocket.
                for player in gs.players.players_iter() {
                    assert!(player.pocket.is_some());
                }
            }
        }
    }
}
