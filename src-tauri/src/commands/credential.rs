use tauri::State;

use crate::crypto;
use crate::error::AppResult;
use crate::models::credential::{Credential, CredentialInput, PasswordUpdate};
use crate::repository::credentials;
use crate::state::AppState;

/// Writes slow credential operations to `<data_dir>/rack42-perf.log` so an
/// intermittent save stall can be traced without a debugger attached.
fn log_slow(state: &AppState, operation: &str, elapsed: std::time::Duration) {
    if elapsed < std::time::Duration::from_millis(500) {
        return;
    }

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or_default();
    let line = format!("{stamp} {operation} {elapsed:?}\n");

    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(state.data_dir.join("rack42-perf.log"))
    {
        use std::io::Write;
        let _ = file.write_all(line.as_bytes());
    }
}

#[tauri::command]
pub async fn list_credentials(
    state: State<'_, AppState>,
    device_id: i64,
) -> AppResult<Vec<Credential>> {
    let started = std::time::Instant::now();
    let result = credentials::list_by_device(&state.db, device_id).await;
    log_slow(&state, "list_credentials", started.elapsed());
    result
}

#[tauri::command]
pub async fn list_all_credentials(state: State<'_, AppState>) -> AppResult<Vec<Credential>> {
    let started = std::time::Instant::now();
    let result = credentials::list_all(&state.db).await;
    log_slow(&state, "list_all_credentials", started.elapsed());
    result
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

    let started = std::time::Instant::now();
    let result = credentials::create(&state.db, &input, cipher.as_deref()).await;
    log_slow(&state, "create_credential", started.elapsed());
    result
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

    let started = std::time::Instant::now();
    let result = credentials::update(&state.db, id, &input, &update).await;
    log_slow(&state, "update_credential", started.elapsed());
    result
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
