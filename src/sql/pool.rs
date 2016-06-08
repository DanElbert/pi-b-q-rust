use r2d2;
use rusqlite::{Connection, Error, SQLITE_OPEN_READ_WRITE};
use std::path::Path;

pub type SqlitePool = r2d2::Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

pub struct SqliteConnectionManager {
    in_memory: bool,
    path: Option<String>,
}

impl SqliteConnectionManager {

    pub fn new(database: &str) -> SqliteConnectionManager {
        match database{
            ":memory:" => {
                SqliteConnectionManager {in_memory: true, path: None}
            },
            _ => {
                SqliteConnectionManager {in_memory: false, path: Some(database.to_string())}
           }
        }
    }
}

impl r2d2::ManageConnection for SqliteConnectionManager {
    type Connection = Connection;
    type Error = Error;

    fn connect(&self) -> Result<Connection, Error> {
        if self.in_memory {
            let conn = try!(Connection::open_in_memory());
            Ok(conn)
        } else {
            match self.path {
                Some(ref path) => {
                    let flags = SQLITE_OPEN_READ_WRITE;
                    let path = Path::new(path);
                    let mut conn = try!(Connection::open_with_flags(path, flags));

                    add_trace(&mut conn);

                    Ok(conn)
                },
                None => unreachable!(),
            }
        }
    }

    fn is_valid(&self, conn: &mut Connection) -> Result<(), Error> {
        conn.execute_batch("")
    }

    fn has_broken(&self, _: &mut Connection) -> bool {
        false
    }
}

#[cfg(feature = "db_trace")]
fn add_trace(conn: &mut Connection) {
    fn f(msg: &str) {
        println!("SQL TRACE: [{}]", msg);
    }
    conn.trace(Some(f));
}

#[cfg(not(feature = "db_trace"))]
fn add_trace(_: &mut Connection) {
}
