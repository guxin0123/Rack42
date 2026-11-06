use tauri::State;

use crate::error::AppResult;
use crate::models::os_family::{OsFamily, OsFamilyInput};
use crate::repository::os_families;
use crate::state::AppState;

#[tauri::command]
pub async fn list_os_families(state: State<'_, AppState>) -> AppResult<Vec<OsFamily>> {
    os_families::list(&state.db).await
}

#[tauri::command]
pub async fn create_os_family(
    state: State<'_, AppState>,
    input: OsFamilyInput,
) -> AppResult<OsFamily> {
    os_families::create(&state.db, &input).await
}

#[tauri::command]
pub async fn update_os_family(
    state: State<'_, AppState>,
    id: i64,
    input: OsFamilyInput,
) -> AppResult<OsFamily> {
    os_families::update(&state.db, id, &input).await
}

#[tauri::command]
pub async fn delete_os_family(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    os_families::delete(&state.db, id).await
}
