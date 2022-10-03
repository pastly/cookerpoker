use poker_core::new::{GameError, GameState};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

pub type OpaqueState = String;
pub type OpaqueFilteredState = String;

// /// Formats the sum of two numbers as string.
// #[pyfunction]
// fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//     Ok((a + b).to_string())
// }

struct PyGameError(GameError);

impl From<PyGameError> for PyErr {
    fn from(error: PyGameError) -> Self {
        PyValueError::new_err(error.0.to_string())
    }
}

impl From<GameError> for PyGameError {
    fn from(other: GameError) -> Self {
        Self(other)
    }
}

#[pyfunction]
fn filter_state(opaque_state: OpaqueState, player_id: i32) -> OpaqueFilteredState {
    let state: GameState =
        serde_json::from_str(&opaque_state).expect("Unable to deserialize state");
    let s = state.filter(player_id);
    serde_json::to_string(&s).unwrap()
}

#[pyfunction]
fn new_game_state() -> OpaqueState {
    serde_json::to_string(&GameState::default()).expect("Unable to encode GameState to JSON")
}

#[pyfunction]
fn devonly_reset_state(opaque_state: OpaqueState) -> OpaqueState {
    let mut state: GameState =
        serde_json::from_str(&opaque_state).expect("Unable to deserialize state");
    state.devonly_reset();
    serde_json::to_string(&state).unwrap()
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

/// A Python module implemented in Rust.
#[pymodule]
fn poker_core_py(_py: Python, m: &PyModule) -> PyResult<()> {
    //m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(filter_state, m)?)?;
    m.add_function(wrap_pyfunction!(new_game_state, m)?)?;
    m.add_function(wrap_pyfunction!(devonly_reset_state, m)?)?;
    m.add_function(wrap_pyfunction!(seat_player, m)?)?;
    Ok(())
}
