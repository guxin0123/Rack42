use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use serde::Serialize;
use url::Url;

use crate::error::AppResult;
use crate::repository::devices;
use crate::state::AppState;

/// A short timeout keeps the device list responsive when a host is offline.
const CONNECT_TIMEOUT: Duration = Duration::from_millis(1500);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortProbe {
    pub credential_id: i64,
    pub label: Option<String>,
    pub address: String,
    pub port: i64,
    pub reachable: bool,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePortProbe {
    pub device_id: i64,
    pub probes: Vec<PortProbe>,
    pub min_latency_ms: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct CredentialTarget {
    id: i64,
    label: Option<String>,
    address: Option<String>,
    port: Option<i64>,
    default_port: Option<i64>,
    actions: String,
}

/// Uses the credential address, falling back to the device primary IP.
fn resolve_host(address: &str) -> Option<String> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.contains("://") {
        return Url::parse(trimmed)
            .ok()
            .and_then(|url| url.host_str().map(str::to_string));
    }

    if let Ok(socket) = trimmed.parse::<SocketAddr>() {
        return Some(socket.ip().to_string());
    }

    if let Ok(ip) = trimmed.parse::<IpAddr>() {
        return Some(ip.to_string());
    }

    Some(trimmed.to_string())
}

fn resolve_port(port: Option<i64>, default_port: Option<i64>, actions: &str) -> Option<i64> {
    if let Some(port) = port {
        return Some(port);
    }
    if let Some(port) = default_port {
        return Some(port);
    }

    let actions: Vec<String> = serde_json::from_str(actions).unwrap_or_default();
    match actions.first().map(String::as_str) {
        Some("rdp") => Some(3389),
        Some("web") => Some(443),
        Some("ssh") | Some("sftp") => Some(22),
        _ => None,
    }
}

async fn probe_host(host: String, port: u16) -> Result<f64, String> {
    let started = Instant::now();
    let outcome = tokio::time::timeout(
        CONNECT_TIMEOUT,
        tokio::net::TcpStream::connect((host.as_str(), port)),
    )
    .await;

    match outcome {
        Ok(Ok(stream)) => {
            drop(stream);
            let latency = started.elapsed().as_secs_f64() * 1000.0;
            Ok((latency * 10.0).round() / 10.0)
        }
        Ok(Err(error)) => Err(format!("连接失败：{error}")),
        Err(_) => Err("连接超时".to_string()),
    }
}

pub async fn probe_device_ports(state: &AppState, device_id: i64) -> AppResult<DevicePortProbe> {
    let device = devices::get(&state.db, device_id).await?;

    let rows = sqlx::query_as::<_, CredentialTarget>(
        "SELECT c.id, c.label, c.address, c.port, t.default_port, t.actions \
         FROM credentials c JOIN credential_types t ON t.id = c.type_id \
         WHERE c.device_id = ? ORDER BY c.id",
    )
    .bind(device_id)
    .fetch_all(&state.db)
    .await?;

    let mut probes: Vec<PortProbe> = Vec::new();
    let mut targets: Vec<(usize, String, u16)> = Vec::new();

    for row in rows {
        let address = row
            .address
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| device.ips.first().cloned());

        let Some(address) = address else {
            probes.push(PortProbe {
                credential_id: row.id,
                label: row.label,
                address: String::new(),
                port: 0,
                reachable: false,
                latency_ms: None,
                error: Some("未配置地址，且设备没有主 IP".to_string()),
            });
            continue;
        };

        let Some(host) = resolve_host(&address) else {
            probes.push(PortProbe {
                credential_id: row.id,
                label: row.label,
                address,
                port: 0,
                reachable: false,
                latency_ms: None,
                error: Some("地址格式无效".to_string()),
            });
            continue;
        };

        let Some(port) = resolve_port(row.port, row.default_port, &row.actions) else {
            probes.push(PortProbe {
                credential_id: row.id,
                label: row.label,
                address,
                port: 0,
                reachable: false,
                latency_ms: None,
                error: Some("凭据类型没有可用端口".to_string()),
            });
            continue;
        };

        if !(1..=65535).contains(&port) {
            probes.push(PortProbe {
                credential_id: row.id,
                label: row.label,
                address,
                port,
                reachable: false,
                latency_ms: None,
                error: Some("端口超出范围".to_string()),
            });
            continue;
        }

        let index = probes.len();
        targets.push((index, host, port as u16));
        probes.push(PortProbe {
            credential_id: row.id,
            label: row.label,
            address,
            port,
            reachable: false,
            latency_ms: None,
            error: None,
        });
    }

    let mut tasks = tokio::task::JoinSet::new();
    for (index, host, port) in targets {
        tasks.spawn(async move { (index, probe_host(host, port).await) });
    }

    while let Some(joined) = tasks.join_next().await {
        if let Ok((index, outcome)) = joined {
            match outcome {
                Ok(latency) => {
                    probes[index].reachable = true;
                    probes[index].latency_ms = Some(latency);
                }
                Err(error) => probes[index].error = Some(error),
            }
        }
    }

    let min_latency_ms = probes
        .iter()
        .filter_map(|probe| probe.latency_ms)
        .min_by(f64::total_cmp);

    Ok(DevicePortProbe {
        device_id,
        probes,
        min_latency_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_url_and_plain_targets() {
        assert_eq!(
            resolve_host(" https://10.0.0.5:8443/admin ").as_deref(),
            Some("10.0.0.5")
        );
        assert_eq!(resolve_host("10.0.0.5").as_deref(), Some("10.0.0.5"));
        assert_eq!(resolve_host("[fe80::1]:22").as_deref(), Some("fe80::1"));
        assert_eq!(resolve_host("fe80::1").as_deref(), Some("fe80::1"));
        assert_eq!(resolve_host("   "), None);
    }

    #[test]
    fn resolves_port_by_credential_type_defaults() {
        assert_eq!(resolve_port(Some(2222), Some(22), "[\"ssh\"]"), Some(2222));
        assert_eq!(resolve_port(None, Some(8443), "[\"web\"]"), Some(8443));
        assert_eq!(resolve_port(None, None, "[\"rdp\"]"), Some(3389));
        assert_eq!(resolve_port(None, None, "[\"web\"]"), Some(443));
        assert_eq!(resolve_port(None, None, "[\"ssh\",\"sftp\"]"), Some(22));
        assert_eq!(resolve_port(None, None, "[]"), None);
    }
}
