use std::path::{Path, PathBuf};

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use tauri::Manager;

fn candidates(app: &tauri::AppHandle) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(value) = std::env::var_os("RACK42_DATA_DIR") {
        dirs.push(PathBuf::from(value));
    }

    if let Ok(dir) = app.path().app_local_data_dir() {
        dirs.push(dir);
    }

    if let Ok(dir) = app.path().app_data_dir() {
        dirs.push(dir);
    }

    dirs.push(std::env::temp_dir().join("Rack42"));

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("data"));
        }
    }

    dirs.dedup();
    dirs
}

/// Portable mode keeps both database and WebView2 profile next to the executable.
pub fn portable_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;
    let parent = exe
        .parent()
        .ok_or_else(|| std::io::Error::other("无法定位程序目录"))?;
    Ok(parent.join("data"))
}

fn portable_marker() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;
    let parent = exe
        .parent()
        .ok_or_else(|| std::io::Error::other("无法定位程序目录"))?;
    Ok(parent.join("portable.mode"))
}

pub fn is_portable() -> bool {
    portable_marker()
        .map(|marker| marker.exists())
        .unwrap_or(false)
}

pub fn enable_portable() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = portable_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    std::fs::write(portable_marker()?, "Rack42 portable mode\n")?;
    Ok(())
}

pub fn existing_database(app: &tauri::AppHandle) -> Option<PathBuf> {
    candidates(app)
        .into_iter()
        .find(|dir| dir.join("rack42.db").exists())
}

async fn try_init(data_dir: &Path) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(data_dir)?;

    let db_path = data_dir.join("rack42.db");
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

pub async fn init(
    app: &tauri::AppHandle,
    portable: bool,
) -> Result<(SqlitePool, PathBuf), Box<dyn std::error::Error>> {
    if portable {
        let data_dir = portable_dir()?;
        let pool = try_init(&data_dir).await.map_err(|error| {
            std::io::Error::other(format!(
                "便携数据目录不可用：{}\n{error}",
                data_dir.display()
            ))
        })?;
        eprintln!("[rack42] portable database = {}", data_dir.display());
        return Ok((pool, data_dir));
    }

    let dirs = candidates(app);
    let mut failures = Vec::new();

    // Prefer an existing database, even if it lives in a fallback directory.
    for dir in &dirs {
        if !dir.join("rack42.db").exists() {
            continue;
        }

        match try_init(dir).await {
            Ok(pool) => {
                eprintln!("[rack42] database = {}", dir.display());
                return Ok((pool, dir.clone()));
            }
            Err(error) => {
                eprintln!(
                    "[rack42] existing database unavailable: {} ({error})",
                    dir.display()
                );
                failures.push(format!("{}: {error}", dir.display()));
            }
        }
    }

    for dir in &dirs {
        match try_init(dir).await {
            Ok(pool) => {
                if failures.is_empty() {
                    eprintln!("[rack42] database = {}", dir.display());
                } else {
                    eprintln!(
                        "[rack42] database = {} (fallback after {} failed location(s))",
                        dir.display(),
                        failures.len()
                    );
                }
                return Ok((pool, dir.clone()));
            }
            Err(error) => {
                eprintln!(
                    "[rack42] data directory unavailable: {} ({error})",
                    dir.display()
                );
                failures.push(format!("{}: {error}", dir.display()));
            }
        }
    }

    Err(format!(
        "无法初始化数据库，已尝试以下位置：\n{}",
        failures.join("\n")
    )
    .into())
}
