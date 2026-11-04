use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub id: i64,
    pub device_id: i64,
    pub type_id: i64,
    pub label: Option<String>,
    pub address: Option<String>,
    pub port: Option<i64>,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub has_password: bool,
    pub note: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CredentialRow {
    pub id: i64,
    pub device_id: i64,
    pub type_id: i64,
    pub label: Option<String>,
    pub address: Option<String>,
    pub port: Option<i64>,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub has_password: bool,
    pub note: Option<String>,
    pub is_default: i64,
}

impl From<CredentialRow> for Credential {
    fn from(row: CredentialRow) -> Self {
        Self {
            id: row.id,
            device_id: row.device_id,
            type_id: row.type_id,
            label: row.label,
            address: row.address,
            port: row.port,
            username: row.username,
            domain: row.domain,
            has_password: row.has_password,
            note: row.note,
            is_default: row.is_default != 0,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialInput {
    pub device_id: i64,
    pub type_id: Option<i64>,
    pub label: Option<String>,
    pub address: Option<String>,
    pub port: Option<i64>,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub password: Option<String>,
    pub note: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

pub enum PasswordUpdate {
    Keep,
    Clear,
    Set(String),
}
