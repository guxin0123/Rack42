-- Configurable OS categories with display icons and per-category OS versions.

CREATE TABLE IF NOT EXISTS os_families (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    slug       TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    icon       TEXT NOT NULL DEFAULT 'monitor',
    options    TEXT NOT NULL DEFAULT '[]',
    is_builtin INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);

INSERT INTO os_families (slug, name, icon, options, is_builtin, sort_order)
VALUES
    ('windows', 'Windows', 'monitor', '["Windows Server 2025","Windows Server 2022","Windows Server 2019","Windows Server 2016","Windows 11","Windows 10"]', 1, 10),
    ('linux', 'Linux', 'terminal', '["Ubuntu 24.04 LTS","Ubuntu 22.04 LTS","Debian 12","Rocky Linux 9","AlmaLinux 9","RHEL 9","CentOS Stream 9","openSUSE Leap 15","Arch Linux"]', 1, 20),
    ('other', '其他', 'server', '["VMware ESXi","Proxmox VE","FreeBSD","macOS","网络设备系统","其他"]', 1, 30);
