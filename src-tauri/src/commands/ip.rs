use tauri::State;

use crate::error::AppResult;
use crate::models::ip::IpOverviewEntry;
use crate::repository::ips;
use crate::state::AppState;

#[tauri::command]
pub async fn list_ip_overview(state: State<'_, AppState>) -> AppResult<Vec<IpOverviewEntry>> {
    ips::list(&state.db).await
}
