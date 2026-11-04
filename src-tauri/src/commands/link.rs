use tauri::State;

use crate::error::AppResult;
use crate::models::link::{Link, LinkInput};
use crate::repository::links;
use crate::state::AppState;

#[tauri::command]
pub async fn list_links(state: State<'_, AppState>) -> AppResult<Vec<Link>> {
    links::list(&state.db).await
}

#[tauri::command]
pub async fn create_link(state: State<'_, AppState>, input: LinkInput) -> AppResult<Link> {
    links::create(&state.db, &input).await
}

#[tauri::command]
pub async fn delete_link(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    links::delete(&state.db, id).await
}
