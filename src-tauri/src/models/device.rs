use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub ips: Vec<String>,
    pub mac: Option<String>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub os: Option<String>,
    pub os_family: Option<String>,
    pub status: String,
    pub rack_id: Option<i64>,
    pub rack_position: Option<i64>,
    pub rack_height: i64,
    pub host_id: Option<i64>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DeviceRow {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub mgmt_ips: String,
    pub mac: Option<String>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub os: Option<String>,
    pub os_family: Option<String>,
    pub status: String,
    pub rack_id: Option<i64>,
    pub rack_position: Option<i64>,
    pub rack_height: i64,
    pub host_id: Option<i64>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<DeviceRow> for Device {
    fn from(row: DeviceRow) -> Self {
        let ips = serde_json::from_str::<Vec<String>>(&row.mgmt_ips).unwrap_or_default();
        Self {
            id: row.id,
            name: row.name,
            kind: row.kind,
            ips,
            mac: row.mac,
            vendor: row.vendor,
            model: row.model,
            serial: row.serial,
            os: row.os,
            os_family: row.os_family,
            status: row.status,
            rack_id: row.rack_id,
            rack_position: row.rack_position,
            rack_height: row.rack_height,
            host_id: row.host_id,
            note: row.note,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInput {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub ips: Vec<String>,
    pub mac: Option<String>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub os: Option<String>,
    pub os_family: Option<String>,
    pub status: Option<String>,
    pub rack_id: Option<i64>,
    pub rack_position: Option<i64>,
    pub rack_height: Option<i64>,
    pub host_id: Option<i64>,
    pub note: Option<String>,
}
