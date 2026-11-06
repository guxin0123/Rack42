use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OsFamily {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub icon: String,
    pub options: Vec<String>,
    pub is_builtin: bool,
    pub sort_order: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OsFamilyRow {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub icon: String,
    pub options: String,
    pub is_builtin: i64,
    pub sort_order: i64,
}

impl From<OsFamilyRow> for OsFamily {
    fn from(row: OsFamilyRow) -> Self {
        Self {
            id: row.id,
            slug: row.slug,
            name: row.name,
            icon: row.icon,
            options: serde_json::from_str(&row.options).unwrap_or_default(),
            is_builtin: row.is_builtin != 0,
            sort_order: row.sort_order,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsFamilyInput {
    pub name: String,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub options: Option<Vec<String>>,
    pub sort_order: Option<i64>,
}
