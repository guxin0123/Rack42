use tauri::State;

use crate::error::AppResult;
use crate::models::device::{Device, DeviceInput};
use crate::repository::devices;
use crate::state::AppState;

#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> AppResult<Vec<Device>> {
    devices::list(&state.db).await
}

#[tauri::command]
pub async fn create_device(state: State<'_, AppState>, input: DeviceInput) -> AppResult<Device> {
    devices::create(&state.db, &input).await
}

#[tauri::command]
pub async fn update_device(
    state: State<'_, AppState>,
    id: i64,
    input: DeviceInput,
) -> AppResult<Device> {
    devices::update(&state.db, id, &input).await
}

#[tauri::command]
pub async fn delete_device(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    devices::delete(&state.db, id).await
}
