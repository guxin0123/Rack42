use tauri::State;

use crate::crypto;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::BootstrapState;
use crate::repository::settings;
use crate::state::AppState;

#[tauri::command]
pub async fn get_bootstrap_state(state: State<'_, AppState>) -> AppResult<BootstrapState> {
    let salt = settings::get(&state.db, "master_salt").await?;
    Ok(BootstrapState {
        initialized: salt.is_some(),
        unlocked: state.is_unlocked(),
        data_dir: state.data_dir.to_string_lossy().to_string(),
        portable: db::is_portable(),
    })
}

#[tauri::command]
pub async fn setup_master_password(state: State<'_, AppState>, password: String) -> AppResult<()> {
    if password.chars().count() < 4 {
        return Err(AppError::Validation("主密码至少需要 4 个字符".to_string()));
    }

    if settings::get(&state.db, "master_salt").await?.is_some() {
        return Err(AppError::Validation("主密码已经设置过了".to_string()));
    }

    let salt = crypto::random_bytes::<16>();
    let key = crypto::derive_key(&password, &salt)?;
    let verifier = crypto::fingerprint(&key);

    settings::set(&state.db, "master_salt", &crypto::to_hex(&salt)).await?;
    settings::set(&state.db, "master_verifier", &verifier).await?;

    *state.key.lock().map_err(|_| AppError::Locked)? = Some(key);
    Ok(())
}

#[tauri::command]
pub async fn unlock(state: State<'_, AppState>, password: String) -> AppResult<()> {
    let salt_hex = settings::get(&state.db, "master_salt")
        .await?
        .ok_or(AppError::NotInitialized)?;
    let verifier = settings::get(&state.db, "master_verifier")
        .await?
        .ok_or(AppError::NotInitialized)?;

    let salt = crypto::from_hex(&salt_hex)?;
    let key = crypto::derive_key(&password, &salt)?;

    if crypto::fingerprint(&key) != verifier {
        return Err(AppError::InvalidPassword);
    }

    *state.key.lock().map_err(|_| AppError::Locked)? = Some(key);
    Ok(())
}

#[tauri::command]
pub async fn lock(state: State<'_, AppState>) -> AppResult<()> {
    *state.key.lock().map_err(|_| AppError::Locked)? = None;
    Ok(())
}
