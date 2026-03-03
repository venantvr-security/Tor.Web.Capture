-- Migration V001: Initial Schema

-- Application configuration
CREATE TABLE IF NOT EXISTS app_config (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    value_type TEXT NOT NULL DEFAULT 'string',
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Capture targets
CREATE TABLE IF NOT EXISTS targets (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    capture_screenshot INTEGER NOT NULL DEFAULT 1,
    capture_html INTEGER NOT NULL DEFAULT 1,
    user_agent_type TEXT NOT NULL DEFAULT 'random',
    custom_user_agent TEXT,
    viewport_width INTEGER DEFAULT 1920,
    viewport_height INTEGER DEFAULT 1080,
    wait_after_load_ms INTEGER DEFAULT 2000,
    tags TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_targets_enabled ON targets(enabled);
CREATE INDEX IF NOT EXISTS idx_targets_url ON targets(url);

-- Schedules
CREATE TABLE IF NOT EXISTS schedules (
    id TEXT PRIMARY KEY NOT NULL,
    target_id TEXT NOT NULL,
    cron_expression TEXT NOT NULL,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    enabled INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    next_run_at TEXT,
    run_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (target_id) REFERENCES targets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_schedules_target ON schedules(target_id);
CREATE INDEX IF NOT EXISTS idx_schedules_next_run ON schedules(next_run_at);
CREATE INDEX IF NOT EXISTS idx_schedules_enabled ON schedules(enabled);

-- Captures
CREATE TABLE IF NOT EXISTS captures (
    id TEXT PRIMARY KEY NOT NULL,
    target_id TEXT NOT NULL,
    schedule_id TEXT,
    status TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    duration_ms INTEGER,
    screenshot_path TEXT,
    screenshot_size_bytes INTEGER,
    html_path TEXT,
    html_size_bytes INTEGER,
    page_title TEXT,
    final_url TEXT,
    http_status_code INTEGER,
    tor_circuit_id TEXT,
    exit_node_ip TEXT,
    exit_node_country TEXT,
    user_agent_used TEXT,
    error_message TEXT,
    error_type TEXT,
    gdrive_screenshot_id TEXT,
    gdrive_html_id TEXT,
    gdrive_uploaded_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (target_id) REFERENCES targets(id) ON DELETE CASCADE,
    FOREIGN KEY (schedule_id) REFERENCES schedules(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_captures_target ON captures(target_id);
CREATE INDEX IF NOT EXISTS idx_captures_status ON captures(status);
CREATE INDEX IF NOT EXISTS idx_captures_created ON captures(created_at);
CREATE INDEX IF NOT EXISTS idx_captures_schedule ON captures(schedule_id);

-- User agents
CREATE TABLE IF NOT EXISTS user_agents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    user_agent_string TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    usage_count INTEGER NOT NULL DEFAULT 0,
    last_used_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_user_agents_category ON user_agents(category);
CREATE INDEX IF NOT EXISTS idx_user_agents_enabled ON user_agents(enabled);

-- Google Drive configuration
CREATE TABLE IF NOT EXISTS gdrive_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    auth_type TEXT NOT NULL,
    client_id TEXT,
    client_secret_encrypted TEXT,
    refresh_token_encrypted TEXT,
    service_account_json_encrypted TEXT,
    target_folder_id TEXT,
    auto_upload INTEGER NOT NULL DEFAULT 0,
    upload_screenshots INTEGER NOT NULL DEFAULT 1,
    upload_html INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Operation logs
CREATE TABLE IF NOT EXISTS operation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    message TEXT NOT NULL,
    level TEXT NOT NULL DEFAULT 'info',
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_logs_type ON operation_logs(operation_type);
CREATE INDEX IF NOT EXISTS idx_logs_level ON operation_logs(level);
CREATE INDEX IF NOT EXISTS idx_logs_created ON operation_logs(created_at);

-- Insert default user agents
INSERT OR IGNORE INTO user_agents (name, user_agent_string, category) VALUES
('Shodan', 'Shodan', 'iot_scanner'),
('Shodan Full', 'Mozilla/5.0 (compatible; Shodan; +https://www.shodan.io)', 'iot_scanner'),
('CensysInspect', 'Mozilla/5.0 (compatible; CensysInspect/1.1; +https://about.censys.io/)', 'security_scanner'),
('Censys Scanner', 'censys/1.0', 'security_scanner'),
('ZGrab', 'Mozilla/5.0 zgrab/0.x', 'security_scanner'),
('ZGrab2', 'zgrab2/0.1', 'security_scanner'),
('Masscan', 'masscan/1.3 (https://github.com/robertdavidgraham/masscan)', 'iot_scanner'),
('Nmap NSE', 'Mozilla/5.0 (compatible; Nmap Scripting Engine; https://nmap.org/book/nse.html)', 'security_scanner'),
('BinaryEdge', 'Mozilla/5.0 (compatible; BinaryEdge; +https://www.binaryedge.io)', 'iot_scanner'),
('Project Sonar', 'Sonar Product Survey (https://sonar.labs.rapid7.com)', 'security_scanner'),
('FOFA', 'FOFA', 'iot_scanner'),
('ZoomEye', 'Mozilla/5.0 (compatible; ZoomEye; +https://www.zoomeye.org)', 'iot_scanner'),
('GreyNoise', 'GreyNoise/1.0 (greynoise.io)', 'security_scanner'),
('Shadowserver', 'Mozilla/5.0 (compatible; Shadowserver; +https://shadowserver.org)', 'security_scanner'),
('SecurityTrails', 'Mozilla/5.0 (compatible; SecurityTrails; +https://securitytrails.com)', 'security_scanner'),
('Onyphe', 'Mozilla/5.0 (compatible; ONYPHE; +https://www.onyphe.io)', 'security_scanner'),
('IPinfo', 'Mozilla/5.0 (compatible; IPinfo; +https://ipinfo.io)', 'security_scanner');

-- Insert default configuration
INSERT OR IGNORE INTO app_config (key, value, value_type, description) VALUES
('tor_enabled', 'true', 'bool', 'Enable TOR routing'),
('tor_new_circuit_per_capture', 'true', 'bool', 'New TOR circuit for each capture'),
('capture_storage_path', './data/captures', 'string', 'Capture storage directory'),
('max_concurrent_captures', '3', 'int', 'Maximum concurrent captures'),
('default_timeout_ms', '30000', 'int', 'Default request timeout'),
('chrome_executable_path', '', 'string', 'Chrome/Chromium path (auto-detect if empty)'),
('web_server_port', '8080', 'int', 'Web server port'),
('web_server_bind_address', '127.0.0.1', 'string', 'Web server bind address');
