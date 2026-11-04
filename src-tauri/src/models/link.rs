use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub id: i64,
    pub src_device_id: i64,
    pub src_port: Option<String>,
    pub dst_device_id: i64,
    pub dst_port: Option<String>,
    pub link_type: Option<String>,
    pub speed: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkInput {
    pub src_device_id: i64,
    pub src_port: Option<String>,
    pub dst_device_id: i64,
    pub dst_port: Option<String>,
    pub link_type: Option<String>,
    pub speed: Option<String>,
    pub note: Option<String>,
}
