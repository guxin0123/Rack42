use tauri::State;

use crate::error::AppResult;
use crate::models::layout::LayoutNode;
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
