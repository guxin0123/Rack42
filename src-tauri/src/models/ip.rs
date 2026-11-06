use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpOverviewEntry {
    pub ip: String,
    pub subnet: String,
    pub device_id: i64,
    pub device_name: String,
    /// `device` for a device management IP, `credential` for a credential address.
    pub source: String,
    pub credential_id: Option<i64>,
    pub credential_label: Option<String>,
}
