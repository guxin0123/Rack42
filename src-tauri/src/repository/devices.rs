use std::net::IpAddr;

use sqlx::SqlitePool;

use crate::error::{AppError, AppResult};
use crate::models::device::{Device, DeviceInput, DeviceRow};

const KINDS: &[&str] = &[
    "physical_server",
    "vm",
    "router",
    "switch",
    "firewall",
    "storage",
    "pdu",
    "other",
];

const STATUSES: &[&str] = &["active", "maintenance", "offline", "retired"];
const OS_FAMILIES: &[&str] = &["windows", "linux", "other"];

fn clean(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

const SELECT: &str = "SELECT id, name, kind, mgmt_ips, mac, vendor, model, serial, os, os_family, status, rack_id, rack_position, rack_height, host_id, note, created_at, updated_at FROM devices";

fn normalize_ips(values: &[String]) -> AppResult<Vec<String>> {
    let mut ips: Vec<String> = Vec::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parsed: IpAddr = trimmed
            .parse()
            .map_err(|_| AppError::Validation(format!("无效的 IP 地址：{trimmed}")))?;
        let normalized = parsed.to_string();

        if !ips.contains(&normalized) {
            ips.push(normalized);
        }

        if ips.len() > 16 {
            return Err(AppError::Validation("最多支持 16 个 IP 地址".to_string()));
        }
    }

    Ok(ips)
}

fn normalize_os_family(input: &DeviceInput) -> AppResult<Option<String>> {
    if clean(&input.os).is_none() {
        return Ok(None);
    }

    let family = input.os_family.as_deref().unwrap_or("other");
    if !OS_FAMILIES.contains(&family) {
        return Err(AppError::Validation("未知的操作系统大类".to_string()));
    }

    Ok(Some(family.to_string()))
}

pub async fn list(pool: &SqlitePool) -> AppResult<Vec<Device>> {
    let rows = sqlx::query_as::<_, DeviceRow>(&format!("{SELECT} ORDER BY name COLLATE NOCASE"))
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(Device::from).collect())
}

pub async fn get(pool: &SqlitePool, id: i64) -> AppResult<Device> {
    let row = sqlx::query_as::<_, DeviceRow>(&format!("{SELECT} WHERE id = ?"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Device::from(row))
}

async fn prepare(
    pool: &SqlitePool,
    input: &DeviceInput,
    exclude: Option<i64>,
) -> AppResult<(Vec<String>, Option<String>)> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("设备名称不能为空".to_string()));
    }

    if !KINDS.contains(&input.kind.as_str()) {
        return Err(AppError::Validation("未知的设备类型".to_string()));
    }

    let status = input.status.as_deref().unwrap_or("active");
    if !STATUSES.contains(&status) {
        return Err(AppError::Validation("未知的设备状态".to_string()));
    }

    let height = input.rack_height.unwrap_or(1);
    if !(1..=48).contains(&height) {
        return Err(AppError::Validation(
            "占用 U 数需要在 1 到 48 之间".to_string(),
        ));
    }

    let ips = normalize_ips(&input.ips)?;
    let os_family = normalize_os_family(input)?;

    match input.rack_id {
        Some(rack_id) => {
            let total_u = sqlx::query_scalar::<_, i64>("SELECT total_u FROM racks WHERE id = ?")
                .bind(rack_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::Validation("所选机柜不存在".to_string()))?;

            if let Some(position) = input.rack_position {
                if position < 1 {
                    return Err(AppError::Validation("起始 U 位需要大于 0".to_string()));
                }
                if position + height - 1 > total_u {
                    return Err(AppError::Validation(format!(
                        "设备超出机柜范围：机柜共 {total_u}U"
                    )));
                }

                let overlaps = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM devices \
                     WHERE rack_id = ? AND id != ? AND rack_position IS NOT NULL \
                     AND rack_position < ? AND rack_position + rack_height > ?",
                )
                .bind(rack_id)
                .bind(exclude.unwrap_or(0))
                .bind(position + height)
                .bind(position)
                .fetch_one(pool)
                .await?;

                if overlaps > 0 {
                    return Err(AppError::Validation(
                        "该 U 位区间与现有设备重叠".to_string(),
                    ));
                }
            }
        }
        None => {
            if input.rack_position.is_some() {
                return Err(AppError::Validation(
                    "未指定机柜时不能设置 U 位".to_string(),
                ));
            }
        }
    }

    if let Some(host_id) = input.host_id {
        if exclude == Some(host_id) {
            return Err(AppError::Validation("设备不能将自己设为宿主机".to_string()));
        }
        let exists = sqlx::query_scalar::<_, i64>("SELECT id FROM devices WHERE id = ?")
            .bind(host_id)
            .fetch_optional(pool)
            .await?;
        if exists.is_none() {
            return Err(AppError::Validation("宿主机不存在".to_string()));
        }
    }

    Ok((ips, os_family))
}

pub async fn create(pool: &SqlitePool, input: &DeviceInput) -> AppResult<Device> {
    let (ips, os_family) = prepare(pool, input, None).await?;
    let mgmt_ips = serde_json::to_string(&ips)?;
    let status = input.status.as_deref().unwrap_or("active");

    let result = sqlx::query(
        "INSERT INTO devices \
         (name, kind, mgmt_ips, mac, vendor, model, serial, os, os_family, status, rack_id, rack_position, rack_height, host_id, note) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(input.name.trim())
    .bind(input.kind.trim())
    .bind(&mgmt_ips)
    .bind(clean(&input.mac))
    .bind(clean(&input.vendor))
    .bind(clean(&input.model))
    .bind(clean(&input.serial))
    .bind(clean(&input.os))
    .bind(os_family)
    .bind(status)
    .bind(input.rack_id)
    .bind(input.rack_position)
    .bind(input.rack_height.unwrap_or(1))
    .bind(input.host_id)
    .bind(clean(&input.note))
    .execute(pool)
    .await?;

    get(pool, result.last_insert_rowid()).await
}

pub async fn update(pool: &SqlitePool, id: i64, input: &DeviceInput) -> AppResult<Device> {
    let (ips, os_family) = prepare(pool, input, Some(id)).await?;
    let mgmt_ips = serde_json::to_string(&ips)?;
    let status = input.status.as_deref().unwrap_or("active");

    let result = sqlx::query(
        "UPDATE devices SET name = ?, kind = ?, mgmt_ips = ?, mac = ?, \
         vendor = ?, model = ?, serial = ?, os = ?, os_family = ?, status = ?, rack_id = ?, \
         rack_position = ?, rack_height = ?, host_id = ?, note = ?, updated_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(input.name.trim())
    .bind(input.kind.trim())
    .bind(&mgmt_ips)
    .bind(clean(&input.mac))
    .bind(clean(&input.vendor))
    .bind(clean(&input.model))
    .bind(clean(&input.serial))
    .bind(clean(&input.os))
    .bind(os_family)
    .bind(status)
    .bind(input.rack_id)
    .bind(input.rack_position)
    .bind(input.rack_height.unwrap_or(1))
    .bind(input.host_id)
    .bind(clean(&input.note))
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM devices WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}
