CREATE TABLE projects (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  start TEXT NOT NULL,
  end TEXT NOT NULL,
  sensor1_name TEXT NOT NULL,
  sensor2_name TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX idx_projects_time ON projects(created_at);
