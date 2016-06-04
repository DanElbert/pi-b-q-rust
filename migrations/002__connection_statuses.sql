CREATE TABLE connection_statuses
(
    id INTEGER PRIMARY KEY NOT NULL,
    is_connect INTEGER NOT NULL,
    is_disconnect INTEGER NOT NULL,
    info TEXT,
    created_at TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_conn_sts_time ON connection_statuses(created_at);
