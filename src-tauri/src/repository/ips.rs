use std::net::{IpAddr, SocketAddr};

use sqlx::SqlitePool;
use url::Url;

use crate::error::AppResult;
use crate::models::ip::IpOverviewEntry;

fn subnet_of(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
        }
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            format!(
                "{:x}:{:x}:{:x}:{:x}::/64",
                segments[0], segments[1], segments[2], segments[3]
            )
        }
    }
}

/// Extracts an IP from a free-form credential address: plain address,
/// `host:port`, or a full URL.
fn parse_address(value: &str) -> Option<IpAddr> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.contains("://") {
        let url = Url::parse(trimmed).ok()?;
        return url.host_str()?.parse().ok();
    }

    if let Ok(ip) = trimmed.parse::<IpAddr>() {
        return Some(ip);
    }

    if let Ok(socket) = trimmed.parse::<SocketAddr>() {
        return Some(socket.ip());
    }

    None
}

fn subnet_key(subnet: &str) -> IpAddr {
    subnet
        .split('/')
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED))
}

fn ip_key(value: &str) -> IpAddr {
    value
        .parse()
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED))
}

pub async fn list(pool: &SqlitePool) -> AppResult<Vec<IpOverviewEntry>> {
    let mut entries: Vec<IpOverviewEntry> = Vec::new();

    let devices =
        sqlx::query_as::<_, (i64, String, String)>("SELECT id, name, mgmt_ips FROM devices")
            .fetch_all(pool)
            .await?;

    for (device_id, device_name, mgmt_ips) in devices {
        let ips = serde_json::from_str::<Vec<String>>(&mgmt_ips).unwrap_or_default();
        for value in ips {
            if let Ok(ip) = value.parse::<IpAddr>() {
                entries.push(IpOverviewEntry {
                    ip: ip.to_string(),
                    subnet: subnet_of(&ip),
                    device_id,
                    device_name: device_name.clone(),
                    source: "device".to_string(),
                    credential_id: None,
                    credential_label: None,
                });
            }
        }
    }

    let credentials = sqlx::query_as::<_, (i64, Option<String>, String, i64, String)>(
        "SELECT c.id, c.label, c.address, d.id, d.name \
         FROM credentials c JOIN devices d ON d.id = c.device_id \
         WHERE c.address IS NOT NULL AND trim(c.address) <> ''",
    )
    .fetch_all(pool)
    .await?;

    for (credential_id, label, address, device_id, device_name) in credentials {
        let Some(ip) = parse_address(&address) else {
            continue;
        };
        entries.push(IpOverviewEntry {
            ip: ip.to_string(),
            subnet: subnet_of(&ip),
            device_id,
            device_name,
            source: "credential".to_string(),
            credential_id: Some(credential_id),
            credential_label: label,
        });
    }

    entries.sort_by_key(|entry| {
        (
            subnet_key(&entry.subnet),
            ip_key(&entry.ip),
            entry.device_name.clone(),
        )
    });

    Ok(entries)
}
