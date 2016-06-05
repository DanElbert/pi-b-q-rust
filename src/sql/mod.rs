mod migrations;

use std::path::Path;
use rusqlite::{self, Connection, SQLITE_OPEN_CREATE, SQLITE_OPEN_READ_WRITE};

use super::models;

pub fn insert_connection_status(conn: &Connection, status: &mut models::ConnectionStatus) -> rusqlite::Result<()> {
    try!(conn.execute("INSERT INTO connection_statuses (is_connect, is_disconnect, info, created_at) VALUES ($1, $2, $3, $4)",
                 &[&status.is_connect, &status.is_disconnect, &status.info, &status.created_at]));

     status.id = conn.last_insert_rowid();
     Ok(())
}

pub fn insert_reading(conn: &Connection, reading: &mut models::Reading) -> rusqlite::Result<()> {
    try!(conn.execute("INSERT INTO readings (value1, value2, timestamp) VALUES ($1, $2, $3)",
                 &[&reading.value1, &reading.value2, &reading.timestamp]));

     reading.id = conn.last_insert_rowid();
     Ok(())
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
