use tauri::State;

use crate::error::AppResult;
use crate::probe::{self, DevicePortProbe};
use crate::state::AppState;

#[tauri::command]
pub async fn probe_device_ports(
    state: State<'_, AppState>,
    device_id: i64,
) -> AppResult<DevicePortProbe> {
    probe::probe_device_ports(&state, device_id).await
}
