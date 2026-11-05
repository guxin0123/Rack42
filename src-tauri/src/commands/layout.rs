use tauri::State;

use crate::error::AppResult;
use crate::models::layout::{LayoutNode, RackLayoutNode};
use crate::repository::layout;
use crate::state::AppState;

#[tauri::command]
pub async fn load_layout(state: State<'_, AppState>, view: String) -> AppResult<Vec<LayoutNode>> {
    layout::load(&state.db, &view).await
}

#[tauri::command]
pub async fn save_layout(
    state: State<'_, AppState>,
    view: String,
    nodes: Vec<LayoutNode>,
) -> AppResult<()> {
    layout::save(&state.db, &view, &nodes).await
}

#[tauri::command]
pub async fn load_rack_layout(
    state: State<'_, AppState>,
    view: String,
) -> AppResult<Vec<RackLayoutNode>> {
    layout::load_racks(&state.db, &view).await
}

#[tauri::command]
pub async fn save_rack_layout(
    state: State<'_, AppState>,
    view: String,
    nodes: Vec<RackLayoutNode>,
) -> AppResult<()> {
    layout::save_racks(&state.db, &view, &nodes).await
}
