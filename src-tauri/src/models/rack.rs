use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Rack {
    pub id: i64,
    pub name: String,
    pub location: Option<String>,
    pub total_u: i64,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RackInput {
    pub name: String,
    pub location: Option<String>,
    pub total_u: Option<i64>,
    pub note: Option<String>,
}
