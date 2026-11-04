use std::path::PathBuf;
use std::sync::Mutex;

use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};

pub struct AppState {
    pub db: SqlitePool,
    pub data_dir: PathBuf,
    pub key: Mutex<Option<[u8; 32]>>,
}

impl AppState {
    pub fn require_key(&self) -> AppResult<[u8; 32]> {
        self.key
            .lock()
            .map_err(|_| AppError::Locked)?
            .ok_or(AppError::Locked)
    }

    pub fn is_unlocked(&self) -> bool {
        self.key.lock().map(|key| key.is_some()).unwrap_or(false)
    }
}
