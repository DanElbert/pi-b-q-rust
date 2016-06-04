CREATE TABLE readings (
  id INTEGER PRIMARY KEY NOT NULL,
  value1 REAL,
  value2 REAL,
  timestamp TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_readings_time ON readings(timestamp);
