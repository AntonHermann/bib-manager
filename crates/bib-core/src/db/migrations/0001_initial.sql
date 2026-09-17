-- Schema version 1: Zotero libraries and sources, sync runs, review queue, LLM call log (spec §5, §6, §12).

CREATE TABLE library (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('user', 'group')),
    -- 0 for the local user library (addressed as /users/0 by the local API), the group id otherwise.
    zotero_id INTEGER NOT NULL,
    name TEXT NOT NULL DEFAULT '',
    UNIQUE (kind, zotero_id)
);

CREATE TABLE source (
    id INTEGER PRIMARY KEY,
    library_id INTEGER NOT NULL REFERENCES library (id),
    item_key TEXT NOT NULL,
    citation_key TEXT,
    item_type TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    year INTEGER,
    doi TEXT,
    -- Zotero item `data` object as JSON; `data_hash` is SHA-256 of its canonical form without `version`.
    data TEXT NOT NULL,
    data_hash TEXT NOT NULL,
    date_modified TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'retired')),
    first_seen TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    last_seen TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE (library_id, item_key)
);
CREATE INDEX source_citation_key ON source (citation_key);

CREATE TABLE source_tag (
    source_id INTEGER NOT NULL REFERENCES source (id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    PRIMARY KEY (source_id, tag)
);

CREATE TABLE collection (
    id INTEGER PRIMARY KEY,
    library_id INTEGER NOT NULL REFERENCES library (id),
    collection_key TEXT NOT NULL,
    name TEXT NOT NULL,
    parent_key TEXT,
    UNIQUE (library_id, collection_key)
);

CREATE TABLE source_collection (
    source_id INTEGER NOT NULL REFERENCES source (id) ON DELETE CASCADE,
    collection_id INTEGER NOT NULL REFERENCES collection (id) ON DELETE CASCADE,
    PRIMARY KEY (source_id, collection_id)
);

CREATE TABLE attachment (
    id INTEGER PRIMARY KEY,
    library_id INTEGER NOT NULL REFERENCES library (id),
    item_key TEXT NOT NULL,
    -- NULL for standalone attachments without a parent source.
    source_id INTEGER REFERENCES source (id),
    content_type TEXT NOT NULL,
    link_mode TEXT NOT NULL,
    -- NULL when Zotero has no local file.
    path TEXT,
    -- Filled by extraction (plan 3); reset when the path changes.
    checksum TEXT,
    page_count INTEGER,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'retired')),
    UNIQUE (library_id, item_key)
);
CREATE INDEX attachment_source ON attachment (source_id);

CREATE TABLE sync_run (
    id INTEGER PRIMARY KEY,
    library_id INTEGER NOT NULL REFERENCES library (id),
    started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    finished_at TEXT,
    status TEXT NOT NULL DEFAULT 'running' CHECK (status IN ('running', 'ok', 'failed')),
    error TEXT,
    sources_new INTEGER NOT NULL DEFAULT 0,
    sources_changed INTEGER NOT NULL DEFAULT 0,
    sources_retired INTEGER NOT NULL DEFAULT 0,
    sources_reactivated INTEGER NOT NULL DEFAULT 0,
    attachments INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE review_queue (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    -- Node convention (spec §5): table name plus row id of the affected entry.
    node_type TEXT,
    node_id INTEGER,
    message TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '{}',
    dedupe_key TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    resolved_at TEXT,
    resolution TEXT
);
-- The same open problem is queued only once; after resolution it may be queued again.
CREATE UNIQUE INDEX review_queue_open_dedupe ON review_queue (dedupe_key) WHERE resolved_at IS NULL;

-- Empty in this version; every later LLM feature must log here (spec §12).
CREATE TABLE llm_call (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    purpose TEXT NOT NULL,
    model TEXT NOT NULL,
    model_version TEXT NOT NULL,
    parameters TEXT NOT NULL,
    prompt TEXT NOT NULL,
    input_checksum TEXT NOT NULL,
    policy_checksum TEXT NOT NULL,
    output TEXT NOT NULL,
    cloud INTEGER NOT NULL CHECK (cloud IN (0, 1)),
    -- For cloud calls: which of the user's own text left the device.
    data_sent TEXT
);
