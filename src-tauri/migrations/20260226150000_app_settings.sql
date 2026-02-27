CREATE TABLE IF NOT EXISTS app_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    online_interval_sec INTEGER NOT NULL DEFAULT 10,
    offline_interval_sec INTEGER NOT NULL DEFAULT 2,
    autostart_enabled BOOLEAN NOT NULL DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO app_settings (id, online_interval_sec, offline_interval_sec, autostart_enabled)
VALUES (1, 10, 2, 0)
ON CONFLICT(id) DO NOTHING;
