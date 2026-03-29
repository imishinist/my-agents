CREATE TABLE IF NOT EXISTS epics (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'draft',
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS features (
    id              TEXT PRIMARY KEY,
    epic_id         TEXT NOT NULL REFERENCES epics(id),
    title           TEXT NOT NULL,
    specification   TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    worker_id       TEXT,
    branch_name     TEXT,
    pr_number       INTEGER,
    depends_on      TEXT NOT NULL DEFAULT '[]',
    priority        TEXT NOT NULL DEFAULT 'medium',
    blocked_reason  TEXT,
    context_envelope TEXT,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_features_epic_id ON features(epic_id);
CREATE INDEX IF NOT EXISTS idx_features_status ON features(status);
CREATE INDEX IF NOT EXISTS idx_features_worker_id ON features(worker_id);

CREATE TABLE IF NOT EXISTS workers (
    id                TEXT PRIMARY KEY,
    runtime_type      TEXT NOT NULL,
    status            TEXT NOT NULL DEFAULT 'idle',
    current_feature_id TEXT,
    worktree_path     TEXT,
    pid               INTEGER,
    last_heartbeat    TEXT,
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS progress_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id  TEXT NOT NULL REFERENCES features(id),
    worker_id   TEXT NOT NULL REFERENCES workers(id),
    step        INTEGER NOT NULL,
    total_steps INTEGER NOT NULL,
    current_task TEXT NOT NULL,
    files_modified TEXT NOT NULL DEFAULT '[]',
    timestamp   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_progress_log_feature_id ON progress_log(feature_id);

CREATE TABLE IF NOT EXISTS escalations (
    id                TEXT PRIMARY KEY,
    feature_id        TEXT NOT NULL REFERENCES features(id),
    escalation_type   TEXT NOT NULL,
    title             TEXT NOT NULL,
    context           TEXT NOT NULL,
    question          TEXT NOT NULL,
    options           TEXT NOT NULL DEFAULT '[]',
    pm_recommendation TEXT,
    pm_reasoning      TEXT,
    status            TEXT NOT NULL DEFAULT 'pending',
    po_answer         TEXT,
    po_notes          TEXT,
    urgency           TEXT NOT NULL DEFAULT 'medium',
    blocking_features TEXT NOT NULL DEFAULT '[]',
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    answered_at       TEXT
);

CREATE INDEX IF NOT EXISTS idx_escalations_status ON escalations(status);
CREATE INDEX IF NOT EXISTS idx_escalations_feature_id ON escalations(feature_id);

CREATE TABLE IF NOT EXISTS reviews (
    id          TEXT PRIMARY KEY,
    feature_id  TEXT NOT NULL REFERENCES features(id),
    pr_number   INTEGER NOT NULL,
    reviewer    TEXT NOT NULL,
    verdict     TEXT NOT NULL,
    summary     TEXT,
    comments    TEXT NOT NULL DEFAULT '[]',
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_reviews_feature_id ON reviews(feature_id);

CREATE TABLE IF NOT EXISTS permissions (
    id          TEXT PRIMARY KEY,
    worker_id   TEXT NOT NULL REFERENCES workers(id),
    feature_id  TEXT NOT NULL REFERENCES features(id),
    action      TEXT NOT NULL,
    category    TEXT NOT NULL,
    reason      TEXT NOT NULL,
    risk_level  TEXT NOT NULL DEFAULT 'low',
    status      TEXT NOT NULL DEFAULT 'pending',
    decided_by  TEXT,
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    decided_at  TEXT
);

CREATE INDEX IF NOT EXISTS idx_permissions_status ON permissions(status);
CREATE INDEX IF NOT EXISTS idx_permissions_worker_id ON permissions(worker_id);

CREATE TABLE IF NOT EXISTS messages (
    id              TEXT PRIMARY KEY,
    correlation_id  TEXT,
    source          TEXT NOT NULL,
    destination     TEXT NOT NULL,
    type            TEXT NOT NULL,
    payload         TEXT NOT NULL,
    timestamp       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_messages_type ON messages(type);
CREATE INDEX IF NOT EXISTS idx_messages_correlation_id ON messages(correlation_id);
CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);

CREATE TABLE IF NOT EXISTS project_memory (
    key         TEXT PRIMARY KEY,
    category    TEXT NOT NULL,
    content     TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_project_memory_category ON project_memory(category);
