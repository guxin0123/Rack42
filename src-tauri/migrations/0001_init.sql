-- Rack42 initial schema

CREATE TABLE IF NOT EXISTS racks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    location    TEXT,
    total_u     INTEGER NOT NULL DEFAULT 42,
    note        TEXT
);

CREATE TABLE IF NOT EXISTS devices (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT NOT NULL,
    kind          TEXT NOT NULL,
    mgmt_ip       TEXT,
    config_addr   TEXT,
    mac           TEXT,
    vendor        TEXT,
    model         TEXT,
    serial        TEXT,
    os            TEXT,
    status        TEXT NOT NULL DEFAULT 'active',
    rack_id       INTEGER REFERENCES racks(id) ON DELETE SET NULL,
    rack_position INTEGER,
    rack_height   INTEGER NOT NULL DEFAULT 1,
    host_id       INTEGER REFERENCES devices(id) ON DELETE SET NULL,
    note          TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS credentials (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id   INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    label       TEXT,
    username    TEXT,
    password    TEXT,
    note        TEXT
);

CREATE TABLE IF NOT EXISTS links (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    src_device_id INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    src_port      TEXT,
    dst_device_id INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    dst_port      TEXT,
    link_type     TEXT,
    speed         TEXT,
    note          TEXT
);

CREATE TABLE IF NOT EXISTS topology_layout (
    view_name   TEXT NOT NULL,
    device_id   INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    x           REAL NOT NULL,
    y           REAL NOT NULL,
    PRIMARY KEY (view_name, device_id)
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_devices_rack ON devices(rack_id);
CREATE INDEX IF NOT EXISTS idx_credentials_device ON credentials(device_id);
CREATE INDEX IF NOT EXISTS idx_links_src ON links(src_device_id);
CREATE INDEX IF NOT EXISTS idx_links_dst ON links(dst_device_id);
