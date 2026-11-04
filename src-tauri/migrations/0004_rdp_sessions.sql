-- Phase 2: pending RDP sessions for cleanup after crashes/restarts.

CREATE TABLE IF NOT EXISTS rdp_sessions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    target      TEXT NOT NULL,
    temp_path   TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
