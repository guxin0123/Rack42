use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sqlx::SqlitePool;
use url::Url;
use zeroize::{Zeroize, Zeroizing};

use crate::crypto;
use crate::error::{AppError, AppResult};
use crate::repository::{credential_types, credentials, devices};
use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConnectionSummary {
    pub device_id: i64,
    pub types: Vec<ConnectionTypeSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTypeSummary {
    pub type_id: i64,
    pub slug: String,
    pub name: String,
    pub icon: String,
    pub actions: Vec<String>,
    pub credential_count: i64,
    pub default_credential_id: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct ConnectionRow {
    device_id: i64,
    type_id: i64,
    slug: String,
    name: String,
    icon: String,
    actions: String,
    credential_count: i64,
    default_credential_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResult {
    pub action: String,
    pub target: String,
}

pub async fn list_device_connections(pool: &SqlitePool) -> AppResult<Vec<DeviceConnectionSummary>> {
    let rows = sqlx::query_as::<_, ConnectionRow>(
        "SELECT c.device_id, t.id AS type_id, t.slug, t.name, t.icon, t.actions, \
                COUNT(*) AS credential_count, \
                COALESCE(MAX(CASE WHEN c.is_default = 1 THEN c.id END), MIN(c.id)) AS default_credential_id \
         FROM credentials c \
         JOIN credential_types t ON t.id = c.type_id \
         GROUP BY c.device_id, t.id \
         ORDER BY c.device_id, t.sort_order, t.id",
    )
    .fetch_all(pool)
    .await?;

    let mut result: Vec<DeviceConnectionSummary> = Vec::new();

    for row in rows {
        let actions = serde_json::from_str::<Vec<String>>(&row.actions).unwrap_or_default();
        let item = ConnectionTypeSummary {
            type_id: row.type_id,
            slug: row.slug,
            name: row.name,
            icon: row.icon,
            actions,
            credential_count: row.credential_count,
            default_credential_id: row.default_credential_id,
        };

        if let Some(last) = result.last_mut() {
            if last.device_id == row.device_id {
                last.types.push(item);
                continue;
            }
        }

        result.push(DeviceConnectionSummary {
            device_id: row.device_id,
            types: vec![item],
        });
    }

    Ok(result)
}

pub async fn cleanup_pending(pool: &SqlitePool) -> AppResult<()> {
    let rows = sqlx::query_as::<_, (i64, String)>("SELECT id, temp_path FROM rdp_sessions")
        .fetch_all(pool)
        .await?;

    for (id, temp_path) in rows {
        match fs::remove_file(&temp_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                eprintln!("[rack42] cannot remove stale RDP file {id}: {error}");
                continue;
            }
        }

        sqlx::query("DELETE FROM rdp_sessions WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn connect_credential(
    state: &AppState,
    credential_id: i64,
    action: &str,
) -> AppResult<ConnectResult> {
    if action != "rdp" && action != "web" {
        return Err(AppError::Validation(
            "SSH/SFTP 连接将在 Phase 3 支持".to_string(),
        ));
    }

    let credential = credentials::get(&state.db, credential_id).await?;
    let credential_type = credential_types::get(&state.db, credential.type_id).await?;

    if !credential_type.actions.iter().any(|item| item == action) {
        return Err(AppError::Validation(format!(
            "凭据类型「{}」不支持该操作",
            credential_type.name
        )));
    }

    let device = devices::get(&state.db, credential.device_id).await?;

    let address = credential
        .address
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| device.ips.first().cloned())
        .ok_or_else(|| AppError::Validation("未配置地址，且设备没有主 IP".to_string()))?;

    let port = credential
        .port
        .or(credential_type.default_port)
        .unwrap_or(if action == "rdp" { 3389 } else { 443 });

    if !(1..=65535).contains(&port) {
        return Err(AppError::Validation(
            "端口需要在 1 到 65535 之间".to_string(),
        ));
    }

    if action == "web" {
        let url = build_web_url(&address, port)?;
        open::that(url.as_str()).map_err(AppError::Io)?;
        return Ok(ConnectResult {
            action: action.to_string(),
            target: url,
        });
    }

    let password = if credential.has_password {
        let cipher = credentials::cipher_for(&state.db, credential_id).await?;
        let key = state.require_key()?;
        Some(Zeroizing::new(crypto::decrypt(&key, &cipher)?))
    } else {
        None
    };

    let host = normalize_host(&address)?;
    let target = format!("{host}:{port}");
    let rdp_path = new_rdp_path(&state.data_dir)?;

    let username = credential.username.as_deref().unwrap_or("").trim();
    let domain = credential.domain.as_deref().unwrap_or("").trim();

    if let Err(error) = write_rdp_file(
        &rdp_path,
        &host,
        port,
        username,
        domain,
        password.as_ref().map(|value| value.as_str()),
    ) {
        let _ = fs::remove_file(&rdp_path);
        return Err(error);
    }

    let temp_path = rdp_path.to_string_lossy().to_string();
    let session = sqlx::query("INSERT INTO rdp_sessions (target, temp_path) VALUES (?, ?)")
        .bind(&target)
        .bind(&temp_path)
        .execute(&state.db)
        .await?;

    let session_id = session.last_insert_rowid();

    let mut child = match Command::new("mstsc.exe").arg(&rdp_path).spawn() {
        Ok(child) => child,
        Err(error) => {
            let _ = fs::remove_file(&rdp_path);
            let _ = sqlx::query("DELETE FROM rdp_sessions WHERE id = ?")
                .bind(session_id)
                .execute(&state.db)
                .await;
            return Err(AppError::Io(error));
        }
    };

    let pool = state.db.clone();
    let cleanup_path = rdp_path.clone();

    std::thread::spawn(move || {
        let _ = child.wait();
        let _ = fs::remove_file(&cleanup_path);
        let _ = tauri::async_runtime::block_on(
            sqlx::query("DELETE FROM rdp_sessions WHERE id = ?")
                .bind(session_id)
                .execute(&pool),
        );
    });

    Ok(ConnectResult {
        action: action.to_string(),
        target,
    })
}

fn new_rdp_path(data_dir: &Path) -> AppResult<PathBuf> {
    let dir = data_dir.join("rdp");
    fs::create_dir_all(&dir)?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    Ok(dir.join(format!("rack42-{stamp}-{}.rdp", std::process::id())))
}

fn normalize_host(address: &str) -> AppResult<String> {
    let host = address.trim();

    if host.is_empty() || host.len() > 255 || host.chars().any(|ch| ch.is_control() || ch == ' ') {
        return Err(AppError::Validation("RDP 地址格式无效".to_string()));
    }

    if host.starts_with('[') && host.ends_with(']') {
        return Ok(host.to_string());
    }

    if let Ok(ipv6) = host.parse::<std::net::Ipv6Addr>() {
        return Ok(format!("[{ipv6}]"));
    }

    if host.contains(':') {
        return Err(AppError::Validation(
            "RDP 地址不要包含端口，请使用端口字段".to_string(),
        ));
    }

    Ok(host.to_string())
}

fn write_rdp_file(
    path: &Path,
    host: &str,
    port: i64,
    username: &str,
    domain: &str,
    password: Option<&str>,
) -> AppResult<()> {
    for (label, value) in [("用户名", username), ("域", domain)] {
        if value.contains('\r') || value.contains('\n') {
            return Err(AppError::Validation(format!("{label}包含非法字符")));
        }
    }

    let has_password = password.is_some_and(|value| !value.is_empty());
    let mut content = String::new();
    content.push_str(&format!("full address:s:{host}:{port}\r\n"));
    if !username.is_empty() {
        content.push_str(&format!("username:s:{username}\r\n"));
    }
    if !domain.is_empty() {
        content.push_str(&format!("domain:s:{domain}\r\n"));
    }
    if has_password {
        let blob = protect_password(password.unwrap_or_default())?;
        content.push_str(&format!("password 51:b:{}\r\n", hex_encode(&blob)));
    }
    content.push_str(&format!(
        "prompt for credentials:i:{}\r\n",
        if has_password { 0 } else { 1 }
    ));
    content.push_str("authentication level:i:2\r\n");
    content.push_str("enablecredsspsupport:i:1\r\n");

    fs::write(path, content)?;
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push_str(&format!("{byte:02x}"));
    }
    result
}

#[cfg(windows)]
fn protect_password(password: &str) -> AppResult<Vec<u8>> {
    use windows_sys::Win32::Foundation::{LocalFree, HLOCAL};
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let mut plain: Vec<u8> = password
        .encode_utf16()
        .flat_map(|unit| unit.to_le_bytes())
        .collect();

    let input = CRYPT_INTEGER_BLOB {
        cbData: plain.len() as u32,
        pbData: plain.as_mut_ptr(),
    };
    let mut output = CRYPT_INTEGER_BLOB::default();

    let ok = unsafe {
        CryptProtectData(
            &input,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };

    plain.zeroize();

    if ok == 0 {
        return Err(AppError::Io(std::io::Error::last_os_error()));
    }

    let encrypted =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe {
        LocalFree(output.pbData as HLOCAL);
    }

    Ok(encrypted)
}

#[cfg(not(windows))]
fn protect_password(_: &str) -> AppResult<Vec<u8>> {
    Err(AppError::Validation("RDP 连接仅支持 Windows".to_string()))
}

fn build_web_url(address: &str, port: i64) -> AppResult<String> {
    let address = address.trim();
    if address.is_empty() {
        return Err(AppError::Validation("未配置 Web 地址".to_string()));
    }

    let candidate = if address.contains("://") {
        address.to_string()
    } else {
        format!("https://{address}")
    };

    let mut url = Url::parse(&candidate)
        .map_err(|error| AppError::Validation(format!("无效的 Web 地址：{error}")))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(AppError::Validation(
            "只允许打开 http/https 地址".to_string(),
        ));
    }

    if url.port().is_none() {
        let port =
            u16::try_from(port).map_err(|_| AppError::Validation("端口超出范围".to_string()))?;
        url.set_port(Some(port))
            .map_err(|_| AppError::Validation("无法设置 Web 端口".to_string()))?;
    }

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_url_defaults_to_https_and_port() {
        assert_eq!(
            build_web_url("192.168.1.10", 443).unwrap(),
            "https://192.168.1.10/"
        );
        assert_eq!(
            build_web_url("http://example.test/path", 80).unwrap(),
            "http://example.test/path"
        );
        assert!(build_web_url("file:///C:/test", 443).is_err());
    }

    #[test]
    fn rdp_host_rejects_port_and_brackets_ipv6() {
        assert!(normalize_host("host:3389").is_err());
        assert_eq!(normalize_host("fe80::1").unwrap(), "[fe80::1]");
        assert_eq!(normalize_host("server-01").unwrap(), "server-01");
    }

    #[cfg(windows)]
    #[test]
    fn dpapi_password_blob_roundtrips() {
        use windows_sys::Win32::Foundation::{LocalFree, HLOCAL};
        use windows_sys::Win32::Security::Cryptography::{
            CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
        };

        let password = "rack42-dpapi-check";
        let blob = protect_password(password).expect("protect password");

        let input = CRYPT_INTEGER_BLOB {
            cbData: blob.len() as u32,
            pbData: blob.as_ptr() as *mut u8,
        };
        let mut output = CRYPT_INTEGER_BLOB::default();
        let ok = unsafe {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        assert_ne!(ok, 0, "CryptUnprotectData failed");

        let plain =
            unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
        unsafe {
            LocalFree(output.pbData as HLOCAL);
        }

        let utf16: Vec<u16> = plain
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        assert_eq!(String::from_utf16(&utf16).unwrap(), password);
    }
}
