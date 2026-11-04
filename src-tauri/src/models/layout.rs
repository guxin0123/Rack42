use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LayoutNode {
    pub device_id: i64,
    pub x: f64,
    pub y: f64,
}
