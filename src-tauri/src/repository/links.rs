use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::link::{Link, LinkInput};

fn clean(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

const SELECT: &str = "SELECT id, src_device_id, src_port, dst_device_id, dst_port, \
    link_type, speed, note FROM links";

pub async fn list(pool: &SqlitePool) -> AppResult<Vec<Link>> {
    let links = sqlx::query_as::<_, Link>(&format!("{SELECT} ORDER BY id"))
        .fetch_all(pool)
        .await?;
    Ok(links)
}

pub async fn get(pool: &SqlitePool, id: i64) -> AppResult<Link> {
    sqlx::query_as::<_, Link>(&format!("{SELECT} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create(pool: &SqlitePool, input: &LinkInput) -> AppResult<Link> {
    if input.src_device_id == input.dst_device_id {
        return Err(AppError::Validation("不能连接同一台设备".to_string()));
    }

    if let Some(link_type) = input.link_type.as_deref() {
        if !["ethernet", "fiber", "power", "mgmt"].contains(&link_type) {
            return Err(AppError::Validation("未知的连接类型".to_string()));
        }
    }

    for device_id in [input.src_device_id, input.dst_device_id] {
        let exists = sqlx::query_scalar::<_, i64>("SELECT id FROM devices WHERE id = ?")
            .bind(device_id)
            .fetch_optional(pool)
            .await?;
        if exists.is_none() {
            return Err(AppError::Validation("连接中的设备不存在".to_string()));
        }
    }

    let result = sqlx::query(
        "INSERT INTO links (src_device_id, src_port, dst_device_id, dst_port, link_type, speed, note) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(input.src_device_id)
    .bind(clean(&input.src_port))
    .bind(input.dst_device_id)
    .bind(clean(&input.dst_port))
    .bind(clean(&input.link_type))
    .bind(clean(&input.speed))
    .bind(clean(&input.note))
    .execute(pool)
    .await?;

    get(pool, result.last_insert_rowid()).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM links WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}
