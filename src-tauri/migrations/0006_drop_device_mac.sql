-- Phase 6: MAC is no longer collected or stored.
-- No historical data or users exist, so the column is dropped outright.

ALTER TABLE devices DROP COLUMN mac;
