use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialType {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub icon: String,
    pub actions: Vec<String>,
    pub default_port: Option<i64>,
    pub is_builtin: bool,
    pub sort_order: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CredentialTypeRow {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub icon: String,
    pub actions: String,
    pub default_port: Option<i64>,
    pub is_builtin: i64,
    pub sort_order: i64,
}

impl From<CredentialTypeRow> for CredentialType {
    fn from(row: CredentialTypeRow) -> Self {
        let actions = serde_json::from_str::<Vec<String>>(&row.actions).unwrap_or_default();
        Self {
            id: row.id,
            slug: row.slug,
            name: row.name,
            icon: row.icon,
            actions,
            default_port: row.default_port,
            is_builtin: row.is_builtin != 0,
            sort_order: row.sort_order,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialTypeInput {
    pub name: String,
    pub slug: Option<String>,
    pub icon: Option<String>,
    #[serde(default)]
    pub actions: Vec<String>,
    pub default_port: Option<i64>,
    pub sort_order: Option<i64>,
}
