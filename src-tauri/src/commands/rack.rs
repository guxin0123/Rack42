use tauri::State;

use crate::error::AppResult;
use crate::models::rack::{Rack, RackInput};
use crate::repository::racks;
use crate::state::AppState;

#[tauri::command]
pub async fn list_racks(state: State<'_, AppState>) -> AppResult<Vec<Rack>> {
    racks::list(&state.db).await
}

#[tauri::command]
pub async fn create_rack(state: State<'_, AppState>, input: RackInput) -> AppResult<Rack> {
    racks::create(&state.db, &input).await
}

#[tauri::command]
pub async fn update_rack(state: State<'_, AppState>, id: i64, input: RackInput) -> AppResult<Rack> {
    racks::update(&state.db, id, &input).await
}

#[tauri::command]
pub async fn delete_rack(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    racks::delete(&state.db, id).await
}
