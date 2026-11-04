use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use url::Url;

use crate::error::{AppError, AppResult};
use crate::repository::settings;

const SETTINGS_KEY: &str = "connection";
const SSH_CLIENTS: &[&str] = &["xshell", "putty", "openssh"];
const SFTP_CLIENTS: &[&str] = &["xftp", "psftp", "openssh"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSettings {
    #[serde(default = "default_ssh_client")]
    pub ssh_client: String,
    #[serde(default)]
    pub ssh_path: Option<String>,
    #[serde(default = "default_sftp_client")]
    pub sftp_client: String,
    #[serde(default)]
    pub sftp_path: Option<String>,
    #[serde(default)]
    pub auto_fill_password: bool,
}

fn default_ssh_client() -> String {
    "openssh".to_string()
}

fn default_sftp_client() -> String {
    "openssh".to_string()
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            ssh_client: default_ssh_client(),
            ssh_path: None,
            sftp_client: default_sftp_client(),
            sftp_path: None,
            auto_fill_password: false,
        }
    }
}

pub async fn load(pool: &SqlitePool) -> AppResult<ConnectionSettings> {
    let raw = settings::get(pool, SETTINGS_KEY).await?;
    let Some(raw) = raw else {
        return Ok(ConnectionSettings::default());
    };

    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

pub async fn save(
    pool: &SqlitePool,
    mut input: ConnectionSettings,
) -> AppResult<ConnectionSettings> {
    input.ssh_client = input.ssh_client.trim().to_ascii_lowercase();
    input.sftp_client = input.sftp_client.trim().to_ascii_lowercase();
    input.ssh_path = clean_path(input.ssh_path);
    input.sftp_path = clean_path(input.sftp_path);

    if !SSH_CLIENTS.contains(&input.ssh_client.as_str()) {
        return Err(AppError::Validation("未知的 SSH 客户端".to_string()));
    }
    if !SFTP_CLIENTS.contains(&input.sftp_client.as_str()) {
        return Err(AppError::Validation("未知的 SFTP 客户端".to_string()));
    }

    validate_configured_path(&input.ssh_client, input.ssh_path.as_deref())?;
    let sftp_client_key = if input.sftp_client == "openssh" {
        "openssh-sftp"
    } else {
        &input.sftp_client
    };
    validate_configured_path(sftp_client_key, input.sftp_path.as_deref())?;

    let raw = serde_json::to_string(&input)?;
    settings::set(pool, SETTINGS_KEY, &raw).await?;

    Ok(input)
}

fn clean_path(path: Option<String>) -> Option<String> {
    path.map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn validate_configured_path(client: &str, path: Option<&str>) -> AppResult<()> {
    let Some(path) = path else {
        return Ok(());
    };

    validate_client_path(client, path)
}

pub fn detect_client_path(client: &str) -> Option<String> {
    let (file_name, candidates) = client_candidates(client)?;

    if let Some(path) = find_in_path(file_name) {
        return Some(path.to_string_lossy().to_string());
    }

    candidates
        .into_iter()
        .find(|path| path.is_file())
        .map(|path| path.to_string_lossy().to_string())
}

fn client_candidates(client: &str) -> Option<(&'static str, Vec<PathBuf>)> {
    let program_files = std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
    let program_files_x86 = std::env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));
    let system_root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));

    let candidates = match client {
        "xshell" => vec![
            program_files_x86.join(r"NetSarang\Xshell 7\Xshell.exe"),
            program_files.join(r"NetSarang\Xshell 7\Xshell.exe"),
        ],
        "xftp" => vec![
            program_files_x86.join(r"NetSarang\Xftp 7\Xftp.exe"),
            program_files.join(r"NetSarang\Xftp 7\Xftp.exe"),
        ],
        "putty" => vec![
            program_files.join(r"PuTTY\putty.exe"),
            program_files_x86.join(r"PuTTY\putty.exe"),
        ],
        "psftp" => vec![
            program_files.join(r"PuTTY\psftp.exe"),
            program_files_x86.join(r"PuTTY\psftp.exe"),
        ],
        "openssh" => vec![system_root.join(r"System32\OpenSSH\ssh.exe")],
        "openssh-sftp" => vec![system_root.join(r"System32\OpenSSH\sftp.exe")],
        _ => return None,
    };

    let file_name = match client {
        "xshell" => "Xshell.exe",
        "xftp" => "Xftp.exe",
        "putty" => "putty.exe",
        "psftp" => "psftp.exe",
        "openssh" => "ssh.exe",
        "openssh-sftp" => "sftp.exe",
        _ => return None,
    };

    Some((file_name, candidates))
}

fn find_in_path(file_name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;

    std::env::split_paths(&path)
        .map(|dir| dir.join(file_name))
        .find(|candidate| candidate.is_file())
}

fn validate_client_path(client: &str, path: &str) -> AppResult<()> {
    let Some((expected, _)) = client_candidates(client) else {
        return Err(AppError::Validation("未知的客户端".to_string()));
    };

    let candidate = Path::new(path);
    if !candidate.is_file() {
        return Err(AppError::Validation(format!("客户端路径不存在：{path}")));
    }

    let file_name = candidate
        .file_name()
        .map(|value| value.to_string_lossy())
        .unwrap_or_default();
    if !file_name.eq_ignore_ascii_case(expected) {
        return Err(AppError::Validation(format!("客户端文件名应为 {expected}")));
    }

    Ok(())
}

fn resolve_client(client: &str, configured: Option<&str>) -> AppResult<PathBuf> {
    let path = configured
        .map(str::to_string)
        .or_else(|| detect_client_path(client))
        .ok_or_else(|| {
            AppError::Validation(format!(
                "未找到 {} 客户端，请先在设置中配置路径",
                client_label(client)
            ))
        })?;

    validate_client_path(client, &path)?;
    Ok(PathBuf::from(path))
}

pub fn client_label(client: &str) -> &'static str {
    match client {
        "xshell" => "Xshell",
        "xftp" => "Xftp",
        "putty" => "PuTTY",
        "psftp" => "psftp",
        "openssh" => "Windows OpenSSH",
        "openssh-sftp" => "Windows OpenSSH sftp",
        _ => "远程连接",
    }
}

pub fn test_client(client: &str, configured: Option<&str>) -> AppResult<String> {
    let path = resolve_client(client, configured)?;
    Ok(format!(
        "{} 路径有效：{}",
        client_label(client),
        path.to_string_lossy()
    ))
}

pub fn launch(
    settings: &ConnectionSettings,
    action: &str,
    address: &str,
    port: i64,
    username: &str,
    password: Option<&str>,
) -> AppResult<String> {
    if !(1..=65535).contains(&port) {
        return Err(AppError::Validation(
            "端口需要在 1 到 65535 之间".to_string(),
        ));
    }

    let host = normalize_host(address)?;
    let username = username.trim().to_string();

    if username.contains('\r') || username.contains('\n') {
        return Err(AppError::Validation("用户名包含非法字符".to_string()));
    }
    if username.contains('@') {
        return Err(AppError::Validation(
            "用户名不能包含 @，请分开填写地址".to_string(),
        ));
    }

    let (client, configured, action_kind) = match action {
        "ssh" => (
            settings.ssh_client.as_str(),
            settings.ssh_path.as_deref(),
            "ssh",
        ),
        "sftp" => (
            settings.sftp_client.as_str(),
            settings.sftp_path.as_deref(),
            "sftp",
        ),
        _ => {
            return Err(AppError::Validation("不支持的远程连接动作".to_string()));
        }
    };

    let client_key = if action == "sftp" && client == "openssh" {
        "openssh-sftp"
    } else {
        client
    };

    let program = resolve_client(client_key, configured)?;
    let password = password.filter(|value| !value.is_empty());

    if password.is_some() && username.is_empty() {
        return Err(AppError::Validation(
            "自动填充密码需要填写用户名".to_string(),
        ));
    }

    let spec = build_spec(client, action_kind, &host, port, &username, password)?;

    let mut command = Command::new(&program);
    command.args(&spec.args);

    #[cfg(windows)]
    if spec.new_console {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        command.creation_flags(CREATE_NEW_CONSOLE);
    }

    command.spawn().map_err(AppError::Io)?;

    Ok(format!("{host}:{port}"))
}

struct LaunchSpec {
    args: Vec<String>,
    new_console: bool,
}

fn build_spec(
    client: &str,
    action: &str,
    host: &str,
    port: i64,
    username: &str,
    password: Option<&str>,
) -> AppResult<LaunchSpec> {
    let host = normalize_host(host)?;
    let port = port.to_string();

    match (client, action) {
        ("xshell", "ssh") => Ok(LaunchSpec {
            args: vec![
                "-url".to_string(),
                build_url("ssh", &host, &port, username, password)?,
            ],
            new_console: false,
        }),
        ("xftp", "sftp") => Ok(LaunchSpec {
            args: vec![
                "-url".to_string(),
                build_url("sftp", &host, &port, username, password)?,
            ],
            new_console: false,
        }),
        ("putty", "ssh") => {
            let mut args = vec!["-ssh".to_string(), "-P".to_string(), port];
            if !username.is_empty() {
                args.push("-l".to_string());
                args.push(username.to_string());
            }
            if let Some(password) = password {
                args.push("-pw".to_string());
                args.push(password.to_string());
            }
            args.push(host);
            Ok(LaunchSpec {
                args,
                new_console: false,
            })
        }
        ("psftp", "sftp") => {
            let mut args = vec!["-P".to_string(), port];
            if !username.is_empty() {
                args.push("-l".to_string());
                args.push(username.to_string());
            }
            if let Some(password) = password {
                args.push("-pw".to_string());
                args.push(password.to_string());
            }
            args.push(host);
            Ok(LaunchSpec {
                args,
                new_console: true,
            })
        }
        ("openssh", "ssh") => {
            let mut args = vec!["-p".to_string(), port];
            if !username.is_empty() {
                args.push("-l".to_string());
                args.push(username.to_string());
            }
            args.push(host);
            Ok(LaunchSpec {
                args,
                new_console: true,
            })
        }
        ("openssh", "sftp") => {
            let destination = if username.is_empty() {
                host.clone()
            } else {
                format!("{username}@{host}")
            };
            Ok(LaunchSpec {
                args: vec!["-P".to_string(), port, destination],
                new_console: true,
            })
        }
        _ => Err(AppError::Validation(
            "SSH/SFTP 客户端与动作不匹配".to_string(),
        )),
    }
}

fn build_url(
    scheme: &str,
    host: &str,
    port: &str,
    username: &str,
    password: Option<&str>,
) -> AppResult<String> {
    let mut url = Url::parse(&format!("{scheme}://{host}:{port}"))
        .map_err(|error| AppError::Validation(format!("无法构造连接地址：{error}")))?;

    if !username.is_empty() {
        url.set_username(username)
            .map_err(|_| AppError::Validation("无法设置用户名".to_string()))?;
    }
    if let Some(password) = password {
        url.set_password(Some(password))
            .map_err(|_| AppError::Validation("无法设置密码".to_string()))?;
    }

    Ok(url.to_string())
}

fn normalize_host(address: &str) -> AppResult<String> {
    let host = address.trim();

    if host.is_empty() || host.len() > 255 || host.chars().any(|ch| ch.is_control() || ch == ' ') {
        return Err(AppError::Validation("SSH 地址格式无效".to_string()));
    }

    if host.starts_with('[') && host.ends_with(']') {
        return Ok(host.to_string());
    }

    if let Ok(ipv6) = host.parse::<std::net::Ipv6Addr>() {
        return Ok(format!("[{ipv6}]"));
    }

    if host.contains(':') {
        return Err(AppError::Validation(
            "SSH 地址不要包含端口，请使用端口字段".to_string(),
        ));
    }

    Ok(host.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_defaults_are_safe() {
        let settings = ConnectionSettings::default();
        assert_eq!(settings.ssh_client, "openssh");
        assert_eq!(settings.sftp_client, "openssh");
        assert!(!settings.auto_fill_password);
    }

    #[test]
    fn ssh_url_without_password_has_no_secret() {
        let spec = build_spec("xshell", "ssh", "server-01", 22, "ubuntu", None).unwrap();
        assert_eq!(spec.args[0], "-url");
        assert_eq!(spec.args[1], "ssh://ubuntu@server-01:22");
        assert!(!spec.args.iter().any(|value| value.contains("123")));
    }

    #[test]
    fn ssh_url_with_password_is_encoded() {
        let spec = build_spec(
            "xshell",
            "ssh",
            "server-01",
            22,
            "ubuntu",
            Some("p@ss word:/?#"),
        )
        .unwrap();
        assert!(spec.args[1].contains("p%40ss%20word"));
        assert!(!spec.args[1].contains("p@ss word"));
    }

    #[test]
    fn putty_and_psftp_use_uppercase_port_flag() {
        let putty = build_spec("putty", "ssh", "host", 2222, "ubuntu", None).unwrap();
        assert_eq!(putty.args[0], "-ssh");
        assert_eq!(putty.args[1], "-P");
        assert_eq!(putty.args[2], "2222");

        let psftp = build_spec("psftp", "sftp", "host", 2222, "ubuntu", None).unwrap();
        assert_eq!(psftp.args[0], "-P");
        assert!(psftp.new_console);
    }

    #[test]
    fn openssh_sftp_uses_user_at_host_destination() {
        let spec = build_spec("openssh", "sftp", "fe80::1", 22, "ubuntu", None).unwrap();
        assert_eq!(spec.args, vec!["-P", "22", "ubuntu@[fe80::1]"]);
        assert!(spec.new_console);
    }
}
