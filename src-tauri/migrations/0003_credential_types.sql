-- Credential types, per-credential address/port, default credentials.
-- Phase 1: no historical data, so devices.config_addr is removed directly.

CREATE TABLE IF NOT EXISTS credential_types (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    slug         TEXT NOT NULL UNIQUE,
    name         TEXT NOT NULL,
    icon         TEXT NOT NULL DEFAULT 'key',
    actions      TEXT NOT NULL DEFAULT '[]',
    default_port INTEGER,
    is_builtin   INTEGER NOT NULL DEFAULT 0,
    sort_order   INTEGER NOT NULL DEFAULT 0
);

INSERT INTO credential_types (slug, name, icon, actions, default_port, is_builtin, sort_order)
VALUES
    ('rdp',   'RDP',   'monitor',  '["rdp"]',        3389, 1, 10),
    ('ssh',   'SSH',   'terminal', '["ssh","sftp"]', 22,   1, 20),
    ('idrac', 'iDRAC', 'server',   '["web"]',        443,  1, 30),
    ('esxi',  'ESXi',  'cloud',    '["web"]',        443,  1, 40),
    ('web',   'Web',   'globe',    '["web"]',        443,  1, 50),
    ('other', '其他',  'key',       '[]',            NULL, 1, 60);

ALTER TABLE credentials ADD COLUMN type_id INTEGER REFERENCES credential_types(id);
ALTER TABLE credentials ADD COLUMN address TEXT;
ALTER TABLE credentials ADD COLUMN port INTEGER;
ALTER TABLE credentials ADD COLUMN domain TEXT;
ALTER TABLE credentials ADD COLUMN is_default INTEGER NOT NULL DEFAULT 0;

UPDATE credentials
SET type_id = (SELECT id FROM credential_types WHERE slug = 'other')
WHERE type_id IS NULL;

ALTER TABLE devices DROP COLUMN config_addr;

CREATE INDEX IF NOT EXISTS idx_credentials_type ON credentials(type_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_credentials_default
    ON credentials(device_id, type_id) WHERE is_default = 1;
