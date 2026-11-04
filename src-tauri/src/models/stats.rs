use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub rack_count: i64,
    pub device_count: i64,
    pub credential_count: i64,
    pub link_count: i64,
}
