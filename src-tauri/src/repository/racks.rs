use sqlx::SqlitePool;

use crate::error::{unique_violation, AppError, AppResult};
use crate::models::rack::{Rack, RackInput};

fn clean(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub async fn list(pool: &SqlitePool) -> AppResult<Vec<Rack>> {
    let racks = sqlx::query_as::<_, Rack>(
        "SELECT id, name, location, total_u, note FROM racks ORDER BY name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;
    Ok(racks)
}

pub async fn get(pool: &SqlitePool, id: i64) -> AppResult<Rack> {
    sqlx::query_as::<_, Rack>("SELECT id, name, location, total_u, note FROM racks WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn create(pool: &SqlitePool, input: &RackInput) -> AppResult<Rack> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("机柜名称不能为空".to_string()));
    }

    let total_u = input.total_u.unwrap_or(42);
    if !(1..=48).contains(&total_u) {
        return Err(AppError::Validation(
            "总 U 数需要在 1 到 48 之间".to_string(),
        ));
    }

    let result =
        sqlx::query("INSERT INTO racks (name, location, total_u, note) VALUES (?, ?, ?, ?)")
            .bind(name)
            .bind(clean(&input.location))
            .bind(total_u)
            .bind(clean(&input.note))
            .execute(pool)
            .await
            .map_err(|error| unique_violation(error, "机柜名称已存在"))?;

    get(pool, result.last_insert_rowid()).await
}

pub async fn update(pool: &SqlitePool, id: i64, input: &RackInput) -> AppResult<Rack> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("机柜名称不能为空".to_string()));
    }

    let total_u = input.total_u.unwrap_or(42);
    if !(1..=48).contains(&total_u) {
        return Err(AppError::Validation(
            "总 U 数需要在 1 到 48 之间".to_string(),
        ));
    }

    let result =
        sqlx::query("UPDATE racks SET name = ?, location = ?, total_u = ?, note = ? WHERE id = ?")
            .bind(name)
            .bind(clean(&input.location))
            .bind(total_u)
            .bind(clean(&input.note))
            .bind(id)
            .execute(pool)
            .await
            .map_err(|error| unique_violation(error, "机柜名称已存在"))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM racks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}
