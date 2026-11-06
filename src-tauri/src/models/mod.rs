pub mod credential;
pub mod credential_type;
pub mod device;
pub mod ip;
pub mod layout;
pub mod link;
pub mod os_family;
pub mod rack;
pub mod stats;

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapState {
    pub initialized: bool,
    pub unlocked: bool,
    pub data_dir: String,
    pub portable: bool,
}
