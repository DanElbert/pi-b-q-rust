mod migrations;
pub mod pool;

use chrono::datetime::DateTime;
//use chrono::offset::TimeZone;
use chrono::offset::local::Local;
use std::path::Path;
use std::time::Duration;
use r2d2;
use rusqlite::{self, Connection, SQLITE_OPEN_CREATE, SQLITE_OPEN_READ_WRITE};

use super::models;

pub fn insert_connection_status(conn: &Connection, status: &mut models::ConnectionStatus) -> rusqlite::Result<()> {
    try!(conn.execute("INSERT INTO connection_statuses (is_connect, is_disconnect, info, created_at) VALUES ($1, $2, $3, $4)",
                 &[&status.is_connect, &status.is_disconnect, &status.info, &status.created_at]));

     status.id = conn.last_insert_rowid();
     Ok(())
}

pub fn get_latest_connection_status(conn: &Connection) -> rusqlite::Result<Option<models::ConnectionStatus>> {
    let sql = "SELECT id, is_connect, is_disconnect, info, created_at FROM connection_statuses ORDER BY created_at DESC LIMIT 1";
    let result = conn.query_row(sql, &[], |row| {
        models::ConnectionStatus {
            id: row.get(0),
            is_connect: row.get(1),
            is_disconnect: row.get(2),
            info: row.get(3),
            created_at: row.get(4)
        }
    });

    match result {
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Ok(p) => Ok(Some(p)),
        Err(e) => Err(e)
    }
}

pub fn insert_reading(conn: &Connection, reading: &mut models::Reading) -> rusqlite::Result<()> {
    try!(conn.execute("INSERT INTO readings (value1, value2, timestamp) VALUES ($1, $2, $3)",
                 &[&reading.value1, &reading.value2, &reading.timestamp]));

     reading.id = conn.last_insert_rowid();
     Ok(())
}

pub fn get_project_readings(conn: &Connection, project: &models::Project, after: Option<DateTime<Local>>) -> rusqlite::Result<Vec<models::Reading>> {
    let mut stmt = try!(conn.prepare("SELECT id, value1, value2, timestamp FROM readings WHERE timestamp > $1 AND timestamp < $2 ORDER BY timestamp"));

    let start_date = match after {
        Some(dt) => dt,
        None => project.start
    };

    let reading_iter = try!(stmt.query_map(&[&start_date, &project.end], |row| {
        models::Reading {
            id: row.get(0),
            value1: row.get(1),
            value2: row.get(2),
            timestamp: row.get(3)
        }
    }));

    let mut result = vec![];

    for reading_row in reading_iter {
        let reading = try!(reading_row);
        result.push(reading);
    }

    Ok(result)
}

pub fn insert_project(conn: &Connection, project: &mut models::Project) -> rusqlite::Result<()> {
    try!(conn.execute("INSERT INTO projects (name, start, end, sensor1_name, sensor2_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                 &[&project.name, &project.start, &project.end, &project.sensor1_name, &project.sensor2_name, &project.created_at, &project.updated_at]));

     project.id = conn.last_insert_rowid();
     Ok(())
}

pub fn update_project(conn: &Connection, project: &mut models::Project) -> rusqlite::Result<()> {
    let changed = try!(conn.execute("UPDATE projects SET name = $1, start = $2, end = $3, sensor1_name = $4, sensor2_name = $5, created_at = $6, updated_at = $7 WHERE id = $8",
                 &[&project.name, &project.start, &project.end, &project.sensor1_name, &project.sensor2_name, &project.created_at, &project.updated_at, &project.id]));

    if changed == 1 {
        Ok(())
    } else {
        Err(rusqlite::Error::StatementChangedRows(changed))
    }
}

pub fn get_projects(conn: &Connection) -> rusqlite::Result<Vec<models::Project>> {
    let mut stmt = try!(conn.prepare("SELECT id, name, start, end, sensor1_name, sensor2_name, created_at, updated_at FROM projects ORDER BY created_at DESC"));
    let project_iter = try!(stmt.query_map(&[], |row| {
        models::Project {
            id: row.get(0),
            name: row.get(1),
            start: row.get(2),
            end: row.get(3),
            sensor1_name: row.get(4),
            sensor2_name: row.get(5),
            created_at: row.get(6),
            updated_at: row.get(7)
        }
    }));

    let mut result = vec![];

    for project_row in project_iter {
        let project = try!(project_row);
        result.push(project);
    }

    Ok(result)
}

pub fn get_project(conn: &Connection, id: i64) -> rusqlite::Result<Option<models::Project>> {
    let sql = "SELECT id, name, start, end, sensor1_name, sensor2_name, created_at, updated_at FROM projects WHERE id = $1";
    let result = conn.query_row(sql, &[&id], |row| {
        models::Project {
            id: row.get(0),
            name: row.get(1),
            start: row.get(2),
            end: row.get(3),
            sensor1_name: row.get(4),
            sensor2_name: row.get(5),
            created_at: row.get(6),
            updated_at: row.get(7)
        }
    });

    match result {
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Ok(p) => Ok(Some(p)),
        Err(e) => Err(e)
    }
}

pub fn get_pool(path: &str, size: Option<u32>) -> r2d2::Pool<pool::SqliteConnectionManager> {
    let manager = pool::SqliteConnectionManager::new(path);
    let size = match size {
        Some(s) => s,
        None => 5
    };

    let config = r2d2::Config::builder()
        .pool_size(size)
        .connection_timeout(Duration::from_millis(1000))
        .build();

    ::r2d2::Pool::new(config, manager).unwrap()
}

// returns a Connection
// if migrate is a Some, migrations are run
pub fn get_connection(path: &str, migrate: Option<String>) -> rusqlite::Result<Connection> {

    let mut flags = SQLITE_OPEN_READ_WRITE;

    if migrate.is_some() {
        flags = flags | SQLITE_OPEN_CREATE;
    }

    let path = Path::new(path);
    let conn = try!(Connection::open_with_flags(path, flags));

    match migrate {
        Some(p) => try!(migrations::perform_migration(&conn, &p)),
        None => {}
    }

    Ok(conn)
}
