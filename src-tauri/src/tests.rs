use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::connections;
use crate::crypto;
use crate::models::credential::{CredentialInput, PasswordUpdate};
use crate::models::credential_type::CredentialTypeInput;
use crate::models::device::DeviceInput;
use crate::models::layout::LayoutNode;
use crate::models::link::LinkInput;
use crate::models::rack::RackInput;
use crate::repository::{credential_types, credentials, devices, layout, links, racks};
use crate::ssh;

async fn test_pool_without_migrations() -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .foreign_keys(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("open in-memory database")
}

async fn test_pool() -> SqlitePool {
    let pool = test_pool_without_migrations().await;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations");

    pool
}

fn rack_input(name: &str) -> RackInput {
    RackInput {
        name: name.to_string(),
        location: Some("3F 机房".to_string()),
        total_u: Some(42),
        note: None,
    }
}

fn device_input(
    name: &str,
    rack_id: Option<i64>,
    position: Option<i64>,
    height: i64,
) -> DeviceInput {
    DeviceInput {
        name: name.to_string(),
        kind: "physical_server".to_string(),
        ips: vec!["10.0.0.1".to_string()],
        mac: None,
        vendor: Some("Dell".to_string()),
        model: Some("R750".to_string()),
        serial: None,
        os: Some("Ubuntu 24.04 LTS".to_string()),
        os_family: Some("linux".to_string()),
        status: Some("active".to_string()),
        rack_id,
        rack_position: position,
        rack_height: Some(height),
        host_id: None,
        note: None,
    }
}

#[tokio::test]
async fn file_database_migrations_and_crud_work() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../.cache/test-db")
        .join(format!("{}-{unique}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("rack42.db");

    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .expect("open file database");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrate file database");

    let rack = racks::create(&pool, &rack_input("FILE01"))
        .await
        .expect("create rack in file database");
    assert_eq!(rack.name, "FILE01");
    assert!(path.exists());

    pool.close().await;
    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn multiple_ips_keep_order_and_primary() {
    let pool = test_pool().await;
    let rack = racks::create(&pool, &rack_input("IP01"))
        .await
        .expect("create rack");

    let mut input = device_input("multi-ip", Some(rack.id), Some(1), 1);
    input.ips = vec![
        "10.0.0.2".to_string(),
        "10.0.0.1".to_string(),
        "10.0.0.2".to_string(),
        "fe80::1".to_string(),
    ];

    let created = devices::create(&pool, &input).await.expect("create device");
    assert_eq!(
        created.ips,
        vec![
            "10.0.0.2".to_string(),
            "10.0.0.1".to_string(),
            "fe80::1".to_string()
        ]
    );
    assert_eq!(created.os_family.as_deref(), Some("linux"));

    input.ips = vec!["10.0.0.9".to_string(), "10.0.0.2".to_string()];
    let updated = devices::update(&pool, created.id, &input)
        .await
        .expect("update device");
    assert_eq!(updated.ips[0], "10.0.0.9");

    let mut invalid = device_input("bad-ip", Some(rack.id), Some(2), 1);
    invalid.ips = vec!["999.1.1.1".to_string()];
    assert!(devices::create(&pool, &invalid).await.is_err());

    let mut invalid_family = device_input("bad-os", Some(rack.id), Some(3), 1);
    invalid_family.os_family = Some("solaris".to_string());
    assert!(devices::create(&pool, &invalid_family).await.is_err());
}

#[tokio::test]
async fn migration_converts_legacy_single_ip_and_os() {
    let pool = test_pool_without_migrations().await;

    sqlx::raw_sql(include_str!("../migrations/0001_init.sql"))
        .execute(&pool)
        .await
        .expect("apply initial schema");
    sqlx::query("INSERT INTO racks (name, total_u) VALUES ('LEGACY', 42)")
        .execute(&pool)
        .await
        .expect("insert rack");
    sqlx::query(
        "INSERT INTO devices (name, kind, mgmt_ip, os) \
         VALUES ('legacy-switch', 'switch', '10.1.1.1', 'Windows Server 2019')",
    )
    .execute(&pool)
    .await
    .expect("insert legacy device");

    sqlx::raw_sql(include_str!("../migrations/0002_device_ips_os.sql"))
        .execute(&pool)
        .await
        .expect("upgrade schema");

    sqlx::query(
        "INSERT INTO credentials (device_id, label, username, password) \
         SELECT id, 'legacy', 'root', 'cipher' FROM devices WHERE name = 'legacy-switch'",
    )
    .execute(&pool)
    .await
    .expect("insert legacy credential");

    sqlx::raw_sql(include_str!("../migrations/0003_credential_types.sql"))
        .execute(&pool)
        .await
        .expect("apply credential type schema");

    let row: (String, Option<String>) =
        sqlx::query_as("SELECT mgmt_ips, os_family FROM devices WHERE name = 'legacy-switch'")
            .fetch_one(&pool)
            .await
            .expect("read migrated device");

    assert_eq!(row.0, r#"["10.1.1.1"]"#);
    assert_eq!(row.1.as_deref(), Some("windows"));

    let types = credential_types::list(&pool)
        .await
        .expect("list seeded credential types");
    assert_eq!(types.len(), 6);

    let other_id = types
        .iter()
        .find(|item| item.slug == "other")
        .expect("other type")
        .id;
    let legacy_type: i64 =
        sqlx::query_scalar("SELECT type_id FROM credentials WHERE label = 'legacy'")
            .fetch_one(&pool)
            .await
            .expect("read migrated credential type");
    assert_eq!(legacy_type, other_id);

    let config_addr_columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('devices') WHERE name = 'config_addr'",
    )
    .fetch_one(&pool)
    .await
    .expect("inspect devices columns");
    assert_eq!(config_addr_columns, 0);
}

#[tokio::test]
async fn credential_without_password_is_allowed() {
    let pool = test_pool().await;
    let rack = racks::create(&pool, &rack_input("C02"))
        .await
        .expect("create rack");
    let device = devices::create(
        &pool,
        &device_input("no-password", Some(rack.id), Some(1), 1),
    )
    .await
    .expect("create device");
    let other = credential_types::list(&pool)
        .await
        .expect("list types")
        .into_iter()
        .find(|item| item.slug == "other")
        .expect("other type");

    let created = credentials::create(
        &pool,
        &CredentialInput {
            device_id: device.id,
            type_id: Some(other.id),
            label: Some("待输入".to_string()),
            address: None,
            port: None,
            username: Some("root".to_string()),
            domain: None,
            password: None,
            note: None,
            is_default: false,
        },
        None,
    )
    .await
    .expect("create credential without password");

    assert!(!created.has_password);
    assert!(credentials::cipher_for(&pool, created.id).await.is_err());
}

#[tokio::test]
async fn default_credential_is_unique_per_device_and_type() {
    let pool = test_pool().await;
    let rack = racks::create(&pool, &rack_input("C03"))
        .await
        .expect("create rack");
    let device = devices::create(
        &pool,
        &device_input("default-cred", Some(rack.id), Some(1), 1),
    )
    .await
    .expect("create device");
    let rdp = credential_types::list(&pool)
        .await
        .expect("list types")
        .into_iter()
        .find(|item| item.slug == "rdp")
        .expect("rdp type");

    let input = CredentialInput {
        device_id: device.id,
        type_id: Some(rdp.id),
        label: Some("rdp-a".to_string()),
        address: None,
        port: None,
        username: Some("admin".to_string()),
        domain: None,
        password: None,
        note: None,
        is_default: true,
    };
    let first = credentials::create(&pool, &input, None)
        .await
        .expect("create first default");

    let second = credentials::create(&pool, &input, None)
        .await
        .expect("create second default");

    let list = credentials::list_by_device(&pool, device.id)
        .await
        .expect("list credentials");
    let defaults: Vec<_> = list
        .iter()
        .filter(|item| item.type_id == rdp.id && item.is_default)
        .collect();
    assert_eq!(defaults.len(), 1);
    assert_eq!(defaults[0].id, second.id);
    assert_ne!(defaults[0].id, first.id);
}

#[tokio::test]
async fn deleting_custom_type_reassigns_credentials() {
    let pool = test_pool().await;
    let custom = credential_types::create(
        &pool,
        &CredentialTypeInput {
            name: "Custom Web".to_string(),
            slug: Some("custom-web".to_string()),
            icon: Some("globe".to_string()),
            actions: vec!["web".to_string()],
            default_port: Some(8443),
            sort_order: Some(90),
        },
    )
    .await
    .expect("create custom type");

    let rack = racks::create(&pool, &rack_input("C04"))
        .await
        .expect("create rack");
    let device = devices::create(
        &pool,
        &device_input("custom-cred", Some(rack.id), Some(1), 1),
    )
    .await
    .expect("create device");

    let created = credentials::create(
        &pool,
        &CredentialInput {
            device_id: device.id,
            type_id: Some(custom.id),
            label: Some("custom".to_string()),
            address: Some("10.0.0.99".to_string()),
            port: Some(8443),
            username: None,
            domain: None,
            password: None,
            note: None,
            is_default: true,
        },
        None,
    )
    .await
    .expect("create custom credential");

    assert!(credential_types::delete(&pool, custom.id).await.is_ok());
    let reassigned = credentials::get(&pool, created.id)
        .await
        .expect("read reassigned credential");
    let other = credential_types::list(&pool)
        .await
        .expect("list types")
        .into_iter()
        .find(|item| item.slug == "other")
        .expect("other type");
    assert_eq!(reassigned.type_id, other.id);
    assert!(!reassigned.is_default);

    let rdp = credential_types::list(&pool)
        .await
        .expect("list types")
        .into_iter()
        .find(|item| item.slug == "rdp")
        .expect("rdp type");
    assert!(credential_types::delete(&pool, rdp.id).await.is_err());
}

#[tokio::test]
async fn device_connection_summary_groups_types_and_defaults() {
    let pool = test_pool().await;
    let rack = racks::create(&pool, &rack_input("C05"))
        .await
        .expect("create rack");
    let device = devices::create(
        &pool,
        &device_input("conn-summary", Some(rack.id), Some(1), 1),
    )
    .await
    .expect("create device");

    let types = credential_types::list(&pool).await.expect("list types");
    let rdp = types
        .iter()
        .find(|item| item.slug == "rdp")
        .expect("rdp type");
    let web = types
        .iter()
        .find(|item| item.slug == "web")
        .expect("web type");

    let mut input = CredentialInput {
        device_id: device.id,
        type_id: Some(rdp.id),
        label: Some("primary".to_string()),
        address: None,
        port: None,
        username: Some("admin".to_string()),
        domain: None,
        password: None,
        note: None,
        is_default: true,
    };
    credentials::create(&pool, &input, None)
        .await
        .expect("create rdp primary");
    input.label = Some("secondary".to_string());
    input.is_default = false;
    credentials::create(&pool, &input, None)
        .await
        .expect("create rdp secondary");

    let web_input = CredentialInput {
        device_id: device.id,
        type_id: Some(web.id),
        label: Some("web".to_string()),
        address: Some("https://example.test".to_string()),
        port: Some(443),
        username: None,
        domain: None,
        password: None,
        note: None,
        is_default: true,
    };
    credentials::create(&pool, &web_input, None)
        .await
        .expect("create web credential");

    let summary = connections::list_device_connections(&pool)
        .await
        .expect("list connections");
    let device_summary = summary
        .iter()
        .find(|item| item.device_id == device.id)
        .expect("device summary");
    assert_eq!(device_summary.types.len(), 2);

    let rdp_summary = device_summary
        .types
        .iter()
        .find(|item| item.type_id == rdp.id)
        .expect("rdp summary");
    assert_eq!(rdp_summary.credential_count, 2);
    assert!(rdp_summary.actions.contains(&"rdp".to_string()));
    assert_eq!(rdp_summary.credentials.len(), 2);
    assert!(rdp_summary.credentials[0].is_default);
    assert_eq!(
        rdp_summary.default_credential_id,
        rdp_summary.credentials[0].id
    );
    assert_eq!(rdp_summary.credentials[0].label.as_deref(), Some("primary"));

    let web_summary = device_summary
        .types
        .iter()
        .find(|item| item.type_id == web.id)
        .expect("web summary");
    assert_eq!(web_summary.credential_count, 1);
    assert!(web_summary.actions.contains(&"web".to_string()));
}

#[tokio::test]
async fn audit_logs_list_and_clear_without_secrets() {
    let pool = test_pool().await;

    sqlx::query(
        "INSERT INTO audit_logs (action, device_name, credential_label, credential_type, target, result, detail) \
         VALUES ('rdp', 'srv-01', '管理口', 'RDP', '10.0.0.5:3389', 'success', NULL)",
    )
    .execute(&pool)
    .await
    .expect("insert audit log");

    let logs = connections::list_audit_logs(&pool, 10)
        .await
        .expect("list audit logs");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, "rdp");
    assert_eq!(logs[0].device_name.as_deref(), Some("srv-01"));
    assert_eq!(logs[0].target.as_deref(), Some("10.0.0.5:3389"));
    assert_eq!(logs[0].result, "success");

    connections::clear_audit_logs(&pool)
        .await
        .expect("clear audit logs");
    assert!(connections::list_audit_logs(&pool, 10)
        .await
        .expect("list after clear")
        .is_empty());
}

#[tokio::test]
async fn custom_type_icons_accept_raster_and_reject_svg() {
    let pool = test_pool().await;

    let raster = credential_types::create(
        &pool,
        &CredentialTypeInput {
            name: "Raster Icon".to_string(),
            slug: Some("raster-icon".to_string()),
            icon: Some("data:image/png;base64,iVBORw0KGgo=".to_string()),
            actions: vec![],
            default_port: None,
            sort_order: Some(80),
        },
    )
    .await
    .expect("create raster icon type");
    assert!(raster.icon.starts_with("data:image/png;base64,"));

    let svg = credential_types::create(
        &pool,
        &CredentialTypeInput {
            name: "SVG Icon".to_string(),
            slug: Some("svg-icon".to_string()),
            icon: Some("data:image/svg+xml;base64,PHN2Zz4=".to_string()),
            actions: vec![],
            default_port: None,
            sort_order: Some(81),
        },
    )
    .await;
    assert!(svg.is_err());
}

#[tokio::test]
async fn connection_settings_default_and_roundtrip() {
    let pool = test_pool().await;

    let defaults = ssh::load(&pool).await.expect("load defaults");
    assert_eq!(defaults.ssh_client, "openssh");
    assert_eq!(defaults.sftp_client, "openssh");
    assert!(!defaults.auto_fill_password);
    assert_eq!(defaults.ssh_path, None);

    let saved = ssh::save(
        &pool,
        ssh::ConnectionSettings {
            ssh_client: "putty".to_string(),
            ssh_path: None,
            sftp_client: "psftp".to_string(),
            sftp_path: None,
            auto_fill_password: true,
        },
    )
    .await
    .expect("save settings");

    assert_eq!(saved.ssh_client, "putty");
    assert!(saved.auto_fill_password);

    let loaded = ssh::load(&pool).await.expect("reload settings");
    assert_eq!(loaded.ssh_client, "putty");
    assert_eq!(loaded.sftp_client, "psftp");
    assert!(loaded.auto_fill_password);
}

#[tokio::test]
async fn rdp_cleanup_removes_files_and_rows() {
    let pool = test_pool().await;
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.cache/rdp-cleanup");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let file = dir.join("stale.rdp");
    std::fs::write(&file, "full address:s:example").expect("write temp rdp");

    sqlx::query("INSERT INTO rdp_sessions (target, temp_path) VALUES (?, ?)")
        .bind("example:3389")
        .bind(file.to_string_lossy().to_string())
        .execute(&pool)
        .await
        .expect("insert session");

    connections::cleanup_pending(&pool)
        .await
        .expect("cleanup sessions");

    assert!(!file.exists());
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rdp_sessions")
        .fetch_one(&pool)
        .await
        .expect("count sessions");
    assert_eq!(remaining, 0);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn crypto_roundtrip_and_wrong_key() {
    let salt = crypto::random_bytes::<16>();
    let key = crypto::derive_key("correct-horse-battery", &salt).expect("derive key");
    let cipher = crypto::encrypt(&key, "s3cret-password").expect("encrypt");

    assert_eq!(
        crypto::decrypt(&key, &cipher).expect("decrypt"),
        "s3cret-password"
    );

    let other = crypto::derive_key("wrong-password", &salt).expect("derive other key");
    assert!(crypto::decrypt(&other, &cipher).is_err());
}

#[tokio::test]
async fn rack_and_device_crud_rejects_overlap() {
    let pool = test_pool().await;

    let rack = racks::create(&pool, &rack_input("A01"))
        .await
        .expect("create rack");
    assert_eq!(rack.total_u, 42);

    let first = devices::create(&pool, &device_input("srv-01", Some(rack.id), Some(1), 2))
        .await
        .expect("create device");
    assert_eq!(first.rack_position, Some(1));
    assert_eq!(first.rack_height, 2);

    let overlap = devices::create(&pool, &device_input("srv-02", Some(rack.id), Some(2), 1)).await;
    assert!(overlap.is_err(), "overlapping U range must be rejected");

    let updated = devices::update(
        &pool,
        first.id,
        &device_input("srv-01-renamed", Some(rack.id), Some(37), 2),
    )
    .await
    .expect("move device");
    assert_eq!(updated.name, "srv-01-renamed");
    assert_eq!(updated.rack_position, Some(37));

    devices::delete(&pool, first.id)
        .await
        .expect("delete device");
    assert!(devices::list(&pool).await.expect("list devices").is_empty());

    racks::delete(&pool, rack.id).await.expect("delete rack");
    assert!(racks::list(&pool).await.expect("list racks").is_empty());
}

#[tokio::test]
async fn credential_secret_is_encrypted_and_revealable() {
    let pool = test_pool().await;
    let rack = racks::create(&pool, &rack_input("B01"))
        .await
        .expect("create rack");
    let device = devices::create(&pool, &device_input("esxi-01", Some(rack.id), Some(1), 1))
        .await
        .expect("create device");

    let key = crypto::random_bytes::<32>();
    let cipher = crypto::encrypt(&key, "hunter2").expect("encrypt password");
    let ssh_type = credential_types::list(&pool)
        .await
        .expect("list types")
        .into_iter()
        .find(|item| item.slug == "ssh")
        .expect("ssh type");

    let created = credentials::create(
        &pool,
        &CredentialInput {
            device_id: device.id,
            type_id: Some(ssh_type.id),
            label: Some("iDRAC".to_string()),
            address: Some("10.0.0.5".to_string()),
            port: Some(22),
            username: Some("root".to_string()),
            domain: None,
            password: Some("hunter2".to_string()),
            note: None,
            is_default: true,
        },
        Some(&cipher),
    )
    .await
    .expect("create credential");

    assert!(created.has_password);
    assert!(created.is_default);
    assert_eq!(created.type_id, ssh_type.id);
    assert_eq!(created.address.as_deref(), Some("10.0.0.5"));
    assert_eq!(created.port, Some(22));

    let stored = credentials::cipher_for(&pool, created.id)
        .await
        .expect("read cipher");
    assert_ne!(stored, "hunter2");
    assert_eq!(
        crypto::decrypt(&key, &stored).expect("decrypt stored"),
        "hunter2"
    );

    let listed = credentials::list_by_device(&pool, device.id)
        .await
        .expect("list credentials");
    assert_eq!(listed.len(), 1);
    assert!(listed[0].has_password);

    let updated = credentials::update(
        &pool,
        created.id,
        &CredentialInput {
            device_id: device.id,
            type_id: Some(ssh_type.id),
            label: Some("Web".to_string()),
            address: None,
            port: None,
            username: Some("admin".to_string()),
            domain: Some("EXAMPLE".to_string()),
            password: None,
            note: None,
            is_default: true,
        },
        &PasswordUpdate::Keep,
    )
    .await
    .expect("update metadata only");
    assert_eq!(updated.label.as_deref(), Some("Web"));
    assert!(updated.has_password);
    assert_eq!(updated.address, None);
    assert_eq!(updated.domain.as_deref(), Some("EXAMPLE"));

    credentials::delete(&pool, created.id)
        .await
        .expect("delete credential");
    assert!(credentials::list_all(&pool)
        .await
        .expect("list all")
        .is_empty());
}

#[tokio::test]
async fn links_and_topology_layout_roundtrip() {
    let pool = test_pool().await;
    let rack = racks::create(&pool, &rack_input("C01"))
        .await
        .expect("create rack");
    let left = devices::create(&pool, &device_input("switch-01", Some(rack.id), Some(1), 1))
        .await
        .expect("create left");
    let right = devices::create(&pool, &device_input("switch-02", Some(rack.id), Some(2), 1))
        .await
        .expect("create right");

    let link = links::create(
        &pool,
        &LinkInput {
            src_device_id: left.id,
            src_port: Some("Gi0/1".to_string()),
            dst_device_id: right.id,
            dst_port: Some("Gi0/2".to_string()),
            link_type: Some("ethernet".to_string()),
            speed: Some("10G".to_string()),
            note: None,
        },
    )
    .await
    .expect("create link");
    assert_eq!(links::list(&pool).await.expect("list links").len(), 1);

    layout::save(
        &pool,
        "default",
        &[
            LayoutNode {
                device_id: left.id,
                x: 10.0,
                y: 20.0,
            },
            LayoutNode {
                device_id: right.id,
                x: 30.0,
                y: 40.0,
            },
        ],
    )
    .await
    .expect("save layout");

    let loaded = layout::load(&pool, "default").await.expect("load layout");
    assert_eq!(loaded.len(), 2);

    links::delete(&pool, link.id).await.expect("delete link");
    assert!(links::list(&pool).await.expect("list links").is_empty());
}
