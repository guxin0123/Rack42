use tauri::State;

use crate::crypto;
use crate::error::AppResult;
use crate::models::credential::{Credential, CredentialInput, PasswordUpdate};
use crate::repository::credentials;
use crate::state::AppState;

#[tauri::command]
pub async fn list_credentials(
    state: State<'_, AppState>,
    device_id: i64,
) -> AppResult<Vec<Credential>> {
    credentials::list_by_device(&state.db, device_id).await
}

#[tauri::command]
pub async fn create_credential(
    state: State<'_, AppState>,
    input: CredentialInput,
) -> AppResult<Credential> {
    let password = input.password.as_deref().filter(|value| !value.is_empty());

    let cipher = match password {
        Some(password) => Some(crypto::encrypt(&state.require_key()?, password)?),
        None => None,
    };

    credentials::create(&state.db, &input, cipher.as_deref()).await
}

#[tauri::command]
pub async fn update_credential(
    state: State<'_, AppState>,
    id: i64,
    input: CredentialInput,
) -> AppResult<Credential> {
    let update = match input.password.as_deref() {
        None => PasswordUpdate::Keep,
        Some("") => PasswordUpdate::Clear,
        Some(value) => {
            let key = state.require_key()?;
            PasswordUpdate::Set(crypto::encrypt(&key, value)?)
        }
    };

    credentials::update(&state.db, id, &input, &update).await
}

#[tauri::command]
pub async fn reveal_credential(state: State<'_, AppState>, id: i64) -> AppResult<String> {
    let key = state.require_key()?;
    let cipher = credentials::cipher_for(&state.db, id).await?;
    crypto::decrypt(&key, &cipher)
}

#[tauri::command]
pub async fn delete_credential(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    credentials::delete(&state.db, id).await
}
