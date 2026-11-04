use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use tauri::State;

use crate::error::AppResult;
use crate::models::stats::Stats;
use crate::repository::{credentials, devices, links, racks, stats};
use crate::state::AppState;

#[tauri::command]
pub async fn get_stats(state: State<'_, AppState>) -> AppResult<Stats> {
    stats::get(&state.db).await
}

#[tauri::command]
pub async fn export_data(state: State<'_, AppState>) -> AppResult<String> {
    let racks = racks::list(&state.db).await?;
    let devices = devices::list(&state.db).await?;
    let links = links::list(&state.db).await?;
    let credentials = credentials::list_all(&state.db).await?;

    let exported_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    let bundle = json!({
        "app": "Rack42",
        "version": env!("CARGO_PKG_VERSION"),
        "exportedAt": exported_at,
        "desensitized": true,
        "racks": racks,
        "devices": devices,
        "links": links,
        "credentials": credentials,
    });

    let export_dir = state.data_dir.join("exports");
    std::fs::create_dir_all(&export_dir)?;
    let path = export_dir.join(format!("rack42-export-{exported_at}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(&bundle)?)?;

    Ok(path.to_string_lossy().to_string())
}
