CREATE TABLE metrics (
       id INTEGER PRIMARY KEY,
       method TEXT NOT NULL,
       name TEXT NOT NULL,
       value REAL NOT NULL,
       created_at DATETIME NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW'))
);

CREATE TABLE upload_jobs (
       id INTEGER PRIMARY KEY,
       metric_id INTEGER REFERENCES metrics(id) ON DELETE CASCADE,
       name TEXT NOT NULL,
       done BOOLEAN NOT NULL DEFAULT false,
       num_attempts INTEGER NOT NULL DEFAULT 0,
       created_at DATETIME NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
       updated_at DATETIME NOT NULL DEFAULT (STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW'))
);

CREATE INDEX upload_jobs_done_idx ON upload_jobs (done);
CREATE INDEX upload_jobs_num_attempts_idx ON upload_jobs (num_attempts);
CREATE INDEX upload_jobs_updated_at_idx ON upload_jobs (updated_at);
