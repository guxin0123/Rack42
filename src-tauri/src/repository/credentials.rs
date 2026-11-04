use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::credential::{Credential, CredentialInput, CredentialRow, PasswordUpdate};

fn clean(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn normalize_port(port: Option<i64>) -> AppResult<Option<i64>> {
    match port {
        Some(value) if !(1..=65535).contains(&value) => Err(AppError::Validation(
            "端口需要在 1 到 65535 之间".to_string(),
        )),
        other => Ok(other),
    }
}

async fn resolve_type(pool: &SqlitePool, type_id: Option<i64>) -> AppResult<i64> {
    let id = match type_id {
        Some(id) => id,
        None => {
            sqlx::query_scalar::<_, i64>("SELECT id FROM credential_types WHERE slug = 'other'")
                .fetch_one(pool)
                .await?
        }
    };

    let exists = sqlx::query_scalar::<_, i64>("SELECT id FROM credential_types WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    exists.ok_or_else(|| AppError::Validation("凭据类型不存在".to_string()))
}

const SELECT: &str = "SELECT id, device_id, type_id, label, address, port, username, domain, \
    CASE WHEN password IS NOT NULL AND password <> '' THEN 1 ELSE 0 END AS has_password, note, is_default \
    FROM credentials";

pub async fn list_by_device(pool: &SqlitePool, device_id: i64) -> AppResult<Vec<Credential>> {
    let rows = sqlx::query_as::<_, CredentialRow>(&format!(
        "{SELECT} WHERE device_id = ? ORDER BY type_id, is_default DESC, id"
    ))
    .bind(device_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Credential::from).collect())
}

pub async fn list_all(pool: &SqlitePool) -> AppResult<Vec<Credential>> {
    let rows =
        sqlx::query_as::<_, CredentialRow>(&format!("{SELECT} ORDER BY device_id, type_id, id"))
            .fetch_all(pool)
            .await?;

    Ok(rows.into_iter().map(Credential::from).collect())
}

pub async fn get(pool: &SqlitePool, id: i64) -> AppResult<Credential> {
    let row = sqlx::query_as::<_, CredentialRow>(&format!("{SELECT} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Credential::from(row))
}

pub async fn create(
    pool: &SqlitePool,
    input: &CredentialInput,
    cipher: Option<&str>,
) -> AppResult<Credential> {
    let device_exists = sqlx::query_scalar::<_, i64>("SELECT id FROM devices WHERE id = ?")
        .bind(input.device_id)
        .fetch_optional(pool)
        .await?;
    if device_exists.is_none() {
        return Err(AppError::Validation("设备不存在".to_string()));
    }

    let type_id = resolve_type(pool, input.type_id).await?;
    let port = normalize_port(input.port)?;

    let mut transaction = pool.begin().await?;

    if input.is_default {
        sqlx::query("UPDATE credentials SET is_default = 0 WHERE device_id = ? AND type_id = ?")
            .bind(input.device_id)
            .bind(type_id)
            .execute(&mut *transaction)
            .await?;
    }

    let result = sqlx::query(
        "INSERT INTO credentials \
         (device_id, type_id, label, address, port, username, domain, password, note, is_default) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(input.device_id)
    .bind(type_id)
    .bind(clean(&input.label))
    .bind(clean(&input.address))
    .bind(port)
    .bind(clean(&input.username))
    .bind(clean(&input.domain))
    .bind(cipher)
    .bind(clean(&input.note))
    .bind(if input.is_default { 1 } else { 0 })
    .execute(&mut *transaction)
    .await?;

    let id = result.last_insert_rowid();
    transaction.commit().await?;

    get(pool, id).await
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    input: &CredentialInput,
    password: &PasswordUpdate,
) -> AppResult<Credential> {
    let existing = get(pool, id).await?;
    let device_id = existing.device_id;
    let type_id = resolve_type(pool, input.type_id.or(Some(existing.type_id))).await?;
    let port = normalize_port(input.port)?;

    let mut transaction = pool.begin().await?;

    if input.is_default {
        sqlx::query(
            "UPDATE credentials SET is_default = 0 \
             WHERE device_id = ? AND type_id = ? AND id != ?",
        )
        .bind(device_id)
        .bind(type_id)
        .bind(id)
        .execute(&mut *transaction)
        .await?;
    }

    let is_default = if input.is_default { 1 } else { 0 };

    let result = match password {
        PasswordUpdate::Keep => {
            sqlx::query(
                "UPDATE credentials SET type_id = ?, label = ?, address = ?, port = ?, \
                 username = ?, domain = ?, note = ?, is_default = ? WHERE id = ?",
            )
            .bind(type_id)
            .bind(clean(&input.label))
            .bind(clean(&input.address))
            .bind(port)
            .bind(clean(&input.username))
            .bind(clean(&input.domain))
            .bind(clean(&input.note))
            .bind(is_default)
            .bind(id)
            .execute(&mut *transaction)
            .await?
        }
        PasswordUpdate::Clear => {
            sqlx::query(
                "UPDATE credentials SET type_id = ?, label = ?, address = ?, port = ?, \
                 username = ?, domain = ?, password = NULL, note = ?, is_default = ? WHERE id = ?",
            )
            .bind(type_id)
            .bind(clean(&input.label))
            .bind(clean(&input.address))
            .bind(port)
            .bind(clean(&input.username))
            .bind(clean(&input.domain))
            .bind(clean(&input.note))
            .bind(is_default)
            .bind(id)
            .execute(&mut *transaction)
            .await?
        }
        PasswordUpdate::Set(cipher) => {
            sqlx::query(
                "UPDATE credentials SET type_id = ?, label = ?, address = ?, port = ?, \
                 username = ?, domain = ?, password = ?, note = ?, is_default = ? WHERE id = ?",
            )
            .bind(type_id)
            .bind(clean(&input.label))
            .bind(clean(&input.address))
            .bind(port)
            .bind(clean(&input.username))
            .bind(clean(&input.domain))
            .bind(cipher)
            .bind(clean(&input.note))
            .bind(is_default)
            .bind(id)
            .execute(&mut *transaction)
            .await?
        }
    };

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    transaction.commit().await?;
    get(pool, id).await
}

pub async fn cipher_for(pool: &SqlitePool, id: i64) -> AppResult<String> {
    let cipher = sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(password, '') FROM credentials WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match cipher {
        Some(value) if !value.is_empty() => Ok(value),
        _ => Err(AppError::Validation("该凭据没有可解密的密码".to_string())),
    }
}

pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM credentials WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}
