use tauri::State;

use crate::structs::{ConnectionState, ObsState};

#[tauri::command]
pub fn obs_state(state: State<'_, ObsState>) -> ConnectionState {
    *state.connection.borrow()
}
