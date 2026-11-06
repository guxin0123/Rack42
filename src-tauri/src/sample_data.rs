//! Demo dataset created on first setup when the user opts in.

use sqlx::{Sqlite, Transaction};

use crate::crypto;
use crate::error::AppResult;

const SAMPLE_PASSWORD: &str = "Demo#2026!";

async fn insert_rack(
    transaction: &mut Transaction<'_, Sqlite>,
    name: &str,
    location: &str,
    total_u: i64,
    note: &str,
) -> AppResult<i64> {
    let result =
        sqlx::query("INSERT INTO racks (name, location, total_u, note) VALUES (?, ?, ?, ?)")
            .bind(name)
            .bind(location)
            .bind(total_u)
            .bind(note)
            .execute(&mut **transaction)
            .await?;

    Ok(result.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
async fn insert_device(
    transaction: &mut Transaction<'_, Sqlite>,
    name: &str,
    kind: &str,
    ips: &[&str],
    vendor: &str,
    model: &str,
    serial: &str,
    os: &str,
    os_family: &str,
    status: &str,
    rack_id: Option<i64>,
    rack_position: Option<i64>,
    rack_height: i64,
    host_id: Option<i64>,
    note: &str,
) -> AppResult<i64> {
    let ips_json = serde_json::to_string(ips)?;

    let result = sqlx::query(
        "INSERT INTO devices \
         (name, kind, mgmt_ips, vendor, model, serial, os, os_family, status, \
          rack_id, rack_position, rack_height, host_id, note) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(name)
    .bind(kind)
    .bind(ips_json)
    .bind(vendor)
    .bind(model)
    .bind(serial)
    .bind(os)
    .bind(os_family)
    .bind(status)
    .bind(rack_id)
    .bind(rack_position)
    .bind(rack_height)
    .bind(host_id)
    .bind(note)
    .execute(&mut **transaction)
    .await?;

    Ok(result.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
async fn insert_link(
    transaction: &mut Transaction<'_, Sqlite>,
    src_device_id: i64,
    src_port: Option<&str>,
    dst_device_id: i64,
    dst_port: Option<&str>,
    link_type: &str,
    speed: Option<&str>,
    note: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO links \
         (src_device_id, src_port, dst_device_id, dst_port, link_type, speed, note) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(src_device_id)
    .bind(src_port)
    .bind(dst_device_id)
    .bind(dst_port)
    .bind(link_type)
    .bind(speed)
    .bind(note)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_credential(
    transaction: &mut Transaction<'_, Sqlite>,
    key: &[u8; 32],
    device_id: i64,
    type_slug: &str,
    label: &str,
    username: &str,
    password: Option<&str>,
    address: Option<&str>,
    port: Option<i64>,
    domain: Option<&str>,
    note: &str,
    is_default: bool,
) -> AppResult<()> {
    let type_id = sqlx::query_scalar::<_, i64>("SELECT id FROM credential_types WHERE slug = ?")
        .bind(type_slug)
        .fetch_one(&mut **transaction)
        .await?;

    let cipher = match password {
        Some(value) => Some(crypto::encrypt(key, value)?),
        None => None,
    };

    sqlx::query(
        "INSERT INTO credentials \
         (device_id, type_id, label, address, port, username, domain, password, note, is_default) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(device_id)
    .bind(type_id)
    .bind(label)
    .bind(address)
    .bind(port)
    .bind(username)
    .bind(domain)
    .bind(cipher)
    .bind(note)
    .bind(if is_default { 1 } else { 0 })
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

/// Inserts a small but realistic demo environment: three racks, physical
/// devices, VMs, links and sample credentials. Passwords are encrypted with the
/// freshly created master key.
pub async fn seed(transaction: &mut Transaction<'_, Sqlite>, key: &[u8; 32]) -> AppResult<()> {
    let rack_a01 = insert_rack(transaction, "A01", "一号机房 · A 排", 42, "核心业务机柜").await?;
    let rack_a02 = insert_rack(transaction, "A02", "一号机房 · A 排", 42, "存储与备份机柜").await?;
    let rack_b01 = insert_rack(transaction, "B01", "二号机房 · B 排", 24, "网络与测试机柜").await?;

    let firewall = insert_device(
        transaction,
        "fw-edge-01",
        "firewall",
        &["10.10.10.1"],
        "Fortinet",
        "FortiGate 60F",
        "FG60F-DEMO01",
        "FortiOS 7.4",
        "other",
        "active",
        Some(rack_a01),
        Some(42),
        1,
        None,
        "边界防火墙 · 双线出口",
    )
    .await?;

    let core_switch = insert_device(
        transaction,
        "core-sw-01",
        "switch",
        &["10.10.10.2"],
        "Cisco",
        "Catalyst 9300-48P",
        "FOC-DEMO-SW01",
        "IOS-XE 17.9",
        "other",
        "active",
        Some(rack_a01),
        Some(41),
        1,
        None,
        "核心交换 · 48 口 PoE",
    )
    .await?;

    let esxi_one = insert_device(
        transaction,
        "esxi-01",
        "physical_server",
        &["10.10.10.11", "10.10.20.11"],
        "Dell",
        "PowerEdge R750",
        "DEMO-R750-01",
        "VMware ESXi 8.0",
        "other",
        "active",
        Some(rack_a01),
        Some(38),
        2,
        None,
        "虚拟化宿主机 · 4 台 VM",
    )
    .await?;

    let esxi_two = insert_device(
        transaction,
        "esxi-02",
        "physical_server",
        &["10.10.10.12"],
        "HPE",
        "ProLiant DL380 Gen11",
        "DEMO-DL380-02",
        "VMware ESXi 8.0",
        "other",
        "maintenance",
        Some(rack_a01),
        Some(35),
        2,
        None,
        "虚拟化宿主机 · 维护中",
    )
    .await?;

    let pdu = insert_device(
        transaction,
        "pdu-a01",
        "pdu",
        &["10.10.10.40"],
        "APC",
        "AP8959",
        "DEMO-APC-A01",
        "APC 固件 6.9",
        "other",
        "active",
        Some(rack_a01),
        Some(1),
        1,
        None,
        "机柜 PDU · 8 位计量",
    )
    .await?;

    let nas = insert_device(
        transaction,
        "nas-01",
        "storage",
        &["10.10.10.30"],
        "Synology",
        "RS1221+",
        "DEMO-RS1221",
        "DSM 7.2",
        "other",
        "active",
        Some(rack_a02),
        Some(40),
        2,
        None,
        "主存储 · NFS / SMB",
    )
    .await?;

    let backup = insert_device(
        transaction,
        "backup-01",
        "storage",
        &["10.10.10.31"],
        "QNAP",
        "TS-873A",
        "DEMO-TS873A",
        "QTS 5.1",
        "other",
        "active",
        Some(rack_a02),
        Some(38),
        2,
        None,
        "备份存储 · 每日增量",
    )
    .await?;

    let lab_switch = insert_device(
        transaction,
        "lab-sw-01",
        "switch",
        &["10.10.30.2"],
        "Cisco",
        "Catalyst 1000-24T",
        "DEMO-C1000",
        "IOS 15.2",
        "other",
        "active",
        Some(rack_b01),
        Some(24),
        1,
        None,
        "测试网络接入",
    )
    .await?;

    let lab_pc = insert_device(
        transaction,
        "lab-pc-01",
        "other",
        &["10.10.30.50"],
        "Dell",
        "OptiPlex 7010",
        "DEMO-OPTIPLEX",
        "Windows 11 Pro",
        "windows",
        "offline",
        None,
        None,
        1,
        None,
        "测试机 · 已离线",
    )
    .await?;

    let dc = insert_device(
        transaction,
        "dc-01",
        "vm",
        &["10.10.20.10"],
        "VMware",
        "Virtual Machine",
        "DEMO-VM-DC01",
        "Windows Server 2022",
        "windows",
        "active",
        None,
        None,
        1,
        Some(esxi_one),
        "域控 / DNS",
    )
    .await?;

    let web = insert_device(
        transaction,
        "web-01",
        "vm",
        &["10.10.20.20"],
        "VMware",
        "Virtual Machine",
        "DEMO-VM-WEB01",
        "Ubuntu 24.04 LTS",
        "linux",
        "active",
        None,
        None,
        1,
        Some(esxi_one),
        "Nginx 前端",
    )
    .await?;

    let db = insert_device(
        transaction,
        "db-01",
        "vm",
        &["10.10.20.30"],
        "VMware",
        "Virtual Machine",
        "DEMO-VM-DB01",
        "Debian 12",
        "linux",
        "active",
        None,
        None,
        1,
        Some(esxi_one),
        "PostgreSQL 主库",
    )
    .await?;

    let gitlab = insert_device(
        transaction,
        "gitlab-01",
        "vm",
        &["10.10.20.40"],
        "VMware",
        "Virtual Machine",
        "DEMO-VM-GIT01",
        "Ubuntu 24.04 LTS",
        "linux",
        "active",
        None,
        None,
        1,
        Some(esxi_one),
        "GitLab 代码仓库",
    )
    .await?;

    let _app = insert_device(
        transaction,
        "app-01",
        "vm",
        &["10.10.20.50"],
        "VMware",
        "Virtual Machine",
        "DEMO-VM-APP01",
        "Rocky Linux 9",
        "linux",
        "maintenance",
        None,
        None,
        1,
        Some(esxi_two),
        "业务应用 · 维护中",
    )
    .await?;

    let _vpn = insert_device(
        transaction,
        "vpn-01",
        "vm",
        &["10.10.20.60"],
        "VMware",
        "Virtual Machine",
        "DEMO-VM-VPN01",
        "Ubuntu 22.04 LTS",
        "linux",
        "offline",
        None,
        None,
        1,
        Some(esxi_two),
        "VPN 网关 · 离线",
    )
    .await?;

    insert_link(
        transaction,
        firewall,
        Some("wan1"),
        core_switch,
        Some("Te1/1/1"),
        "fiber",
        Some("10G"),
        "互联网出口",
    )
    .await?;
    insert_link(
        transaction,
        core_switch,
        Some("Te1/0/1"),
        esxi_one,
        Some("vmnic0"),
        "fiber",
        Some("10G"),
        "虚拟化上行",
    )
    .await?;
    insert_link(
        transaction,
        core_switch,
        Some("Te1/0/2"),
        esxi_two,
        Some("vmnic0"),
        "fiber",
        Some("10G"),
        "虚拟化上行",
    )
    .await?;
    insert_link(
        transaction,
        core_switch,
        Some("Te1/0/3"),
        nas,
        Some("eth0"),
        "ethernet",
        Some("10G"),
        "存储网络",
    )
    .await?;
    insert_link(
        transaction,
        core_switch,
        Some("Te1/0/4"),
        backup,
        Some("eth0"),
        "ethernet",
        Some("10G"),
        "备份网络",
    )
    .await?;
    insert_link(
        transaction,
        core_switch,
        Some("Te1/1/1"),
        lab_switch,
        Some("Te1/0/1"),
        "fiber",
        Some("10G"),
        "测试区上联",
    )
    .await?;
    insert_link(
        transaction,
        pdu,
        Some("C13-1"),
        esxi_one,
        Some("PSU1"),
        "power",
        None,
        "主电源",
    )
    .await?;
    insert_link(
        transaction,
        pdu,
        Some("C13-2"),
        esxi_two,
        Some("PSU1"),
        "power",
        None,
        "主电源",
    )
    .await?;
    insert_link(
        transaction,
        lab_switch,
        Some("Gi1/0/1"),
        lab_pc,
        Some("eth0"),
        "ethernet",
        Some("1G"),
        "测试机接入",
    )
    .await?;

    insert_credential(
        transaction,
        key,
        firewall,
        "web",
        "防火墙管理",
        "admin",
        Some(SAMPLE_PASSWORD),
        Some("10.10.10.1"),
        None,
        None,
        "示例密码：Demo#2026!",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        core_switch,
        "ssh",
        "网络运维",
        "netops",
        Some(SAMPLE_PASSWORD),
        Some("10.10.10.2"),
        None,
        None,
        "示例密码：Demo#2026!",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        esxi_one,
        "esxi",
        "vCenter 管理",
        "root",
        Some(SAMPLE_PASSWORD),
        Some("10.10.10.11"),
        None,
        None,
        "示例密码：Demo#2026!",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        esxi_one,
        "ssh",
        "ESXi SSH",
        "root",
        Some(SAMPLE_PASSWORD),
        Some("10.10.10.11"),
        None,
        None,
        "示例密码：Demo#2026!",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        esxi_two,
        "esxi",
        "vCenter 管理",
        "root",
        Some(SAMPLE_PASSWORD),
        Some("10.10.10.12"),
        None,
        None,
        "示例密码：Demo#2026!",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        nas,
        "web",
        "NAS 管理",
        "admin",
        Some(SAMPLE_PASSWORD),
        Some("10.10.10.30"),
        None,
        None,
        "示例密码：Demo#2026!",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        backup,
        "web",
        "备份管理",
        "admin",
        Some(SAMPLE_PASSWORD),
        Some("10.10.10.31"),
        None,
        None,
        "示例密码：Demo#2026!",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        dc,
        "rdp",
        "域管理员",
        "Administrator",
        Some(SAMPLE_PASSWORD),
        Some("10.10.20.10"),
        None,
        Some("CORP"),
        "示例密码：Demo#2026!",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        web,
        "ssh",
        "部署账号",
        "deploy",
        Some(SAMPLE_PASSWORD),
        Some("10.10.20.20"),
        None,
        None,
        "示例密码：Demo#2026!",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        db,
        "ssh",
        "数据库运维",
        "dba",
        Some(SAMPLE_PASSWORD),
        Some("10.10.20.30"),
        None,
        None,
        "示例密码：Demo#2026!",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        gitlab,
        "web",
        "GitLab 管理",
        "root",
        None,
        Some("10.10.20.40"),
        None,
        None,
        "示例：未保存密码",
        true,
    )
    .await?;
    insert_credential(
        transaction,
        key,
        lab_pc,
        "rdp",
        "本地管理员",
        "labadmin",
        Some(SAMPLE_PASSWORD),
        Some("10.10.30.50"),
        None,
        None,
        "示例密码：Demo#2026!",
        true,
    )
    .await?;

    Ok(())
}
