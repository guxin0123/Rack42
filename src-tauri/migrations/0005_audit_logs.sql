-- Phase 5: connection audit trail. Never stores passwords or usernames.
CREATE TABLE IF NOT EXISTS audit_logs (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    action           TEXT NOT NULL,
    device_id        INTEGER,
    device_name      TEXT,
    credential_id    INTEGER,
    credential_label TEXT,
    credential_type  TEXT,
    target           TEXT,
    result           TEXT NOT NULL,       -- success | failed
    detail           TEXT,
    created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_created ON audit_logs(created_at DESC);
