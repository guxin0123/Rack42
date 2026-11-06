use sqlx::SqlitePool;

use crate::error::{unique_violation, AppError, AppResult};
use crate::models::os_family::{OsFamily, OsFamilyInput, OsFamilyRow};

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
        "os".to_string()
    } else {
        slug
    }
}

async fn unique_slug(pool: &SqlitePool, base: &str) -> AppResult<String> {
    let mut candidate = base.to_string();
    let mut suffix = 2;

    loop {
        let exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM os_families WHERE slug = ?")
                .bind(&candidate)
                .fetch_one(pool)
                .await?;

        if exists == 0 {
            return Ok(candidate);
        }

        candidate = format!("{base}-{suffix}");
        suffix += 1;
        if suffix > 100 {
            return Err(AppError::Validation(
                "无法生成唯一的操作系统分类标识".to_string(),
            ));
        }
    }
}

fn normalize_options(values: &[String]) -> AppResult<Vec<String>> {
    let mut options = Vec::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().count() > 80 {
            return Err(AppError::Validation(
                "单个系统名称不能超过 80 个字符".to_string(),
            ));
        }
        if !options.iter().any(|item| item == trimmed) {
            options.push(trimmed.to_string());
        }
        if options.len() > 200 {
            return Err(AppError::Validation(
                "系统列表最多支持 200 个系统".to_string(),
            ));
        }
    }

    Ok(options)
}

fn normalize_icon(icon: Option<&str>) -> AppResult<String> {
    let value = icon
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("monitor");

    if value.starts_with("data:image/") {
        const ALLOWED: &[&str] = &[
            "data:image/png;base64,",
            "data:image/jpeg;base64,",
            "data:image/webp;base64,",
        ];
        if !ALLOWED.iter().any(|prefix| value.starts_with(prefix)) {
            return Err(AppError::Validation(
                "自定义图标只支持 PNG / JPEG / WebP".to_string(),
            ));
        }
        if value.len() > 200_000 {
            return Err(AppError::Validation("自定义图标不能超过 150KB".to_string()));
        }
    } else if value.len() > 64 {
        return Err(AppError::Validation("图标标识过长".to_string()));
    }

    Ok(value.to_string())
}

const SELECT: &str =
    "SELECT id, slug, name, icon, options, is_builtin, sort_order FROM os_families";

pub async fn list(pool: &SqlitePool) -> AppResult<Vec<OsFamily>> {
    let rows = sqlx::query_as::<_, OsFamilyRow>(&format!("{SELECT} ORDER BY sort_order, id"))
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(OsFamily::from).collect())
}

pub async fn get(pool: &SqlitePool, id: i64) -> AppResult<OsFamily> {
    let row = sqlx::query_as::<_, OsFamilyRow>(&format!("{SELECT} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(OsFamily::from(row))
}

pub async fn create(pool: &SqlitePool, input: &OsFamilyInput) -> AppResult<OsFamily> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("操作系统分类名称不能为空".to_string()));
    }

    let base = slugify(
        input
            .slug
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&name),
    );
    let slug = unique_slug(pool, &base).await?;
    let options = normalize_options(input.options.as_deref().unwrap_or(&[]))?;
    let options_json = serde_json::to_string(&options)?;
    let icon = normalize_icon(input.icon.as_deref())?;
    let sort_order = input.sort_order.unwrap_or(0);

    let result = sqlx::query(
        "INSERT INTO os_families (slug, name, icon, options, is_builtin, sort_order) \
         VALUES (?, ?, ?, ?, 0, ?)",
    )
    .bind(&slug)
    .bind(&name)
    .bind(icon)
    .bind(&options_json)
    .bind(sort_order)
    .execute(pool)
    .await
    .map_err(|error| unique_violation(error, "操作系统分类标识已存在"))?;

    get(pool, result.last_insert_rowid()).await
}

pub async fn update(pool: &SqlitePool, id: i64, input: &OsFamilyInput) -> AppResult<OsFamily> {
    let existing = get(pool, id).await?;

    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("操作系统分类名称不能为空".to_string()));
    }

    let icon = match input
        .icon
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => normalize_icon(Some(value))?,
        None => existing.icon.clone(),
    };

    let options = match &input.options {
        Some(values) => normalize_options(values)?,
        None => existing.options.clone(),
    };
    let options_json = serde_json::to_string(&options)?;
    let sort_order = input.sort_order.unwrap_or(existing.sort_order);

    let result = sqlx::query(
        "UPDATE os_families SET name = ?, icon = ?, options = ?, sort_order = ? WHERE id = ?",
    )
    .bind(name)
    .bind(icon)
    .bind(&options_json)
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
        return Err(AppError::Validation("内置操作系统分类不能删除".to_string()));
    }

    let mut transaction = pool.begin().await?;

    sqlx::query("UPDATE devices SET os_family = 'other' WHERE os_family = ?")
        .bind(&existing.slug)
        .execute(&mut *transaction)
        .await?;

    let result = sqlx::query("DELETE FROM os_families WHERE id = ?")
        .bind(id)
        .execute(&mut *transaction)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    transaction.commit().await?;
    Ok(())
}
