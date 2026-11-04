use tauri::State;

use crate::connections::{
    connect_credential as connect, list_device_connections as list, ConnectResult,
    DeviceConnectionSummary,
};
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command]
pub async fn list_device_connections(
    state: State<'_, AppState>,
) -> AppResult<Vec<DeviceConnectionSummary>> {
    list(&state.db).await
}

#[tauri::command]
pub async fn connect_credential(
    state: State<'_, AppState>,
    credential_id: i64,
    action: String,
) -> AppResult<ConnectResult> {
    connect(&state, credential_id, &action).await
}
