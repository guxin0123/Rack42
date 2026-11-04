use sqlx::SqlitePool;

use crate::error::{unique_violation, AppError, AppResult};
use crate::models::credential_type::{CredentialType, CredentialTypeInput, CredentialTypeRow};

const ACTIONS: &[&str] = &["rdp", "ssh", "sftp", "web"];

fn slugify(value: &str) -> String {
    let mut slug = String::new();

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "custom".to_string()
    } else {
        slug
    }
}

fn normalize_actions(input: &[String]) -> AppResult<Vec<String>> {
    let mut actions = Vec::new();

    for action in input {
        let action = action.trim();
        if !ACTIONS.contains(&action) {
            return Err(AppError::Validation(format!("未知的凭据动作：{action}")));
        }
        if !actions.iter().any(|item| item == action) {
            actions.push(action.to_string());
        }
    }

    Ok(actions)
}

fn normalize_port(port: Option<i64>) -> AppResult<Option<i64>> {
    match port {
        Some(value) if !(1..=65535).contains(&value) => Err(AppError::Validation(
            "默认端口需要在 1 到 65535 之间".to_string(),
        )),
        other => Ok(other),
    }
}

fn normalize_icon(icon: Option<&str>) -> String {
    icon.map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("key")
        .to_string()
}

const SELECT: &str = "SELECT id, slug, name, icon, actions, default_port, is_builtin, sort_order FROM credential_types";

pub async fn list(pool: &SqlitePool) -> AppResult<Vec<CredentialType>> {
    let rows = sqlx::query_as::<_, CredentialTypeRow>(&format!("{SELECT} ORDER BY sort_order, id"))
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(CredentialType::from).collect())
}

pub async fn get(pool: &SqlitePool, id: i64) -> AppResult<CredentialType> {
    let row = sqlx::query_as::<_, CredentialTypeRow>(&format!("{SELECT} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(CredentialType::from(row))
}

pub async fn create(pool: &SqlitePool, input: &CredentialTypeInput) -> AppResult<CredentialType> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("凭据类型名称不能为空".to_string()));
    }

    let slug = input
        .slug
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| slugify(&name));

    let actions = normalize_actions(&input.actions)?;
    let actions_json = serde_json::to_string(&actions)?;
    let default_port = normalize_port(input.default_port)?;
    let icon = normalize_icon(input.icon.as_deref());
    let sort_order = input.sort_order.unwrap_or(0);

    let result = sqlx::query(
        "INSERT INTO credential_types (slug, name, icon, actions, default_port, is_builtin, sort_order) \
         VALUES (?, ?, ?, ?, ?, 0, ?)",
    )
    .bind(&slug)
    .bind(&name)
    .bind(icon)
    .bind(&actions_json)
    .bind(default_port)
    .bind(sort_order)
    .execute(pool)
    .await
    .map_err(|error| unique_violation(error, "凭据类型标识已存在"))?;

    get(pool, result.last_insert_rowid()).await
}

pub async fn update(
    pool: &SqlitePool,
    id: i64,
    input: &CredentialTypeInput,
) -> AppResult<CredentialType> {
    let existing = get(pool, id).await?;

    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("凭据类型名称不能为空".to_string()));
    }

    let actions = normalize_actions(&input.actions)?;
    let actions_json = serde_json::to_string(&actions)?;
    let default_port = normalize_port(input.default_port)?;
    let icon = normalize_icon(input.icon.as_deref());
    let sort_order = input.sort_order.unwrap_or(existing.sort_order);

    let result = sqlx::query(
        "UPDATE credential_types SET name = ?, icon = ?, actions = ?, default_port = ?, sort_order = ? \
         WHERE id = ?",
    )
    .bind(name)
    .bind(icon)
    .bind(&actions_json)
    .bind(default_port)
    .bind(sort_order)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let existing = get(pool, id).await?;
    if existing.is_builtin {
        return Err(AppError::Validation("内置凭据类型不能删除".to_string()));
    }

    let other_id =
        sqlx::query_scalar::<_, i64>("SELECT id FROM credential_types WHERE slug = 'other'")
            .fetch_one(pool)
            .await?;

    let mut transaction = pool.begin().await?;

    sqlx::query("UPDATE credentials SET is_default = 0 WHERE type_id = ?")
        .bind(id)
        .execute(&mut *transaction)
        .await?;

    sqlx::query("UPDATE credentials SET type_id = ? WHERE type_id = ?")
        .bind(other_id)
        .bind(id)
        .execute(&mut *transaction)
        .await?;

    sqlx::query("DELETE FROM credential_types WHERE id = ?")
        .bind(id)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;
    Ok(())
}
