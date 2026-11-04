use tauri::State;

use crate::error::AppResult;
use crate::models::credential_type::{CredentialType, CredentialTypeInput};
use crate::repository::credential_types;
use crate::state::AppState;

#[tauri::command]
pub async fn list_credential_types(state: State<'_, AppState>) -> AppResult<Vec<CredentialType>> {
    credential_types::list(&state.db).await
}

#[tauri::command]
pub async fn create_credential_type(
    state: State<'_, AppState>,
    input: CredentialTypeInput,
) -> AppResult<CredentialType> {
    credential_types::create(&state.db, &input).await
}

#[tauri::command]
pub async fn update_credential_type(
    state: State<'_, AppState>,
    id: i64,
    input: CredentialTypeInput,
) -> AppResult<CredentialType> {
    credential_types::update(&state.db, id, &input).await
}

#[tauri::command]
pub async fn delete_credential_type(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    credential_types::delete(&state.db, id).await
}
