use tauri::State;

use crate::connections::{
    connect_credential as connect, list_device_connections as list, ConnectResult,
    DeviceConnectionSummary,
};
use crate::error::AppResult;
use crate::ssh::{self, ConnectionSettings};
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

#[tauri::command]
pub async fn get_connection_settings(state: State<'_, AppState>) -> AppResult<ConnectionSettings> {
    ssh::load(&state.db).await
}

#[tauri::command]
pub async fn update_connection_settings(
    state: State<'_, AppState>,
    input: ConnectionSettings,
) -> AppResult<ConnectionSettings> {
    ssh::save(&state.db, input).await
}

#[tauri::command]
pub fn detect_client_path(client: String) -> AppResult<Option<String>> {
    Ok(ssh::detect_client_path(&client))
}

#[tauri::command]
pub fn test_connection_client(client: String, path: Option<String>) -> AppResult<String> {
    ssh::test_client(&client, path.as_deref())
}
