-- Rack positions in the topology view. Racks are not devices, so they need
-- their own layout table; VM positions remain derived from their host.

CREATE TABLE IF NOT EXISTS topology_rack_layout (
    view_name   TEXT NOT NULL,
    rack_id     INTEGER NOT NULL REFERENCES racks(id) ON DELETE CASCADE,
    x           REAL NOT NULL,
    y           REAL NOT NULL,
    PRIMARY KEY (view_name, rack_id)
);
