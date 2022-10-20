use poker_core::log::LogItem;
use poker_core::state::GameState;
use poker_core::{GameError, PlayerId, SeqNum};
use poker_messages::{action, Msg};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

pub type OpaqueState = String;
pub type OpaqueFilteredState = String;
pub type OpaqueMsg = String;

#[derive(Debug, derive_more::Display)]
enum PyGameError {
    GameError(GameError),
    MessageNotAnAction,
    //GameStateDeserializeError,
}

impl From<PyGameError> for PyErr {
    fn from(error: PyGameError) -> Self {
        PyValueError::new_err(error.to_string())
    }
}

impl From<GameError> for PyGameError {
    fn from(other: GameError) -> Self {
        Self::GameError(other)
    }
}

#[pyfunction]
fn new_game_state() -> OpaqueState {
    serde_json::to_string(&GameState::default()).expect("Unable to encode GameState to JSON")
}

#[pyfunction]
fn seat_player(
    opaque_state: OpaqueState,
    player_id: i32,
    stack: i32,
) -> Result<OpaqueState, PyGameError> {
    let mut state: GameState =
        serde_json::from_str(&opaque_state).expect("Unable to deserialize state");
    state.try_sit(player_id, stack)?;
    Ok(serde_json::to_string(&state).unwrap())
}

#[pyfunction]
fn tick_state(opaque_state: OpaqueState) -> Result<OpaqueState, PyGameError> {
    let mut state: GameState =
        serde_json::from_str(&opaque_state).expect("Unable to deserialize state");
    state.tick()?;
    Ok(serde_json::to_string(&state).unwrap())
}

#[pyfunction]
fn player_action(
    opaque_state: OpaqueState,
    player_id: PlayerId,
    opaque_action: OpaqueMsg,
) -> Result<OpaqueState, PyGameError> {
    let mut state: GameState =
        serde_json::from_str(&opaque_state).expect("Unable to deserialize state");
    let action: Msg =
        serde_json::from_str(&opaque_action).expect("unable to deserialize player action");
    if let Msg::Action(a) = action {
        match a {
            action::Msg::Fold => state.player_folds(player_id)?,
            action::Msg::Call => state.player_calls(player_id)?,
            action::Msg::Check => state.player_checks(player_id)?,
            action::Msg::Bet(v) => state.player_bets(player_id, v)?,
            action::Msg::Raise(v) => state.player_raises(player_id, v)?,
        }
    } else {
        return Err(PyGameError::MessageNotAnAction);
    }
    Ok(serde_json::to_string(&state).unwrap())
}

#[pyfunction]
fn state_changes_since(
    opaque_state: OpaqueState,
    seq_num: SeqNum,
    player_id: PlayerId,
) -> Result<OpaqueMsg, PyGameError> {
    let state: GameState =
        serde_json::from_str(&opaque_state).expect("Unable to deserialize state");
    let changes: Vec<(SeqNum, LogItem)> =
        state.filtered_changes_since(seq_num, player_id).collect();
    Ok(serde_json::to_string(&Msg::GameLogs(changes)).unwrap())
}

/// A Python module implemented in Rust.
#[pymodule]
fn poker_core_py(_py: Python, m: &PyModule) -> PyResult<()> {
    //m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(new_game_state, m)?)?;
    m.add_function(wrap_pyfunction!(seat_player, m)?)?;
    m.add_function(wrap_pyfunction!(tick_state, m)?)?;
    m.add_function(wrap_pyfunction!(player_action, m)?)?;
    m.add_function(wrap_pyfunction!(state_changes_since, m)?)?;
    Ok(())
}
