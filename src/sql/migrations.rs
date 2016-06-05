use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use rusqlite::{self, Connection};

struct Version {
    path: String,
    version: u32
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        self.version == other.version
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        self.version.partial_cmp(&other.version)
    }

}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        self.version.cmp(&other.version)
    }
}

pub fn perform_migration(conn: &Connection, migrate_directory: &str) -> rusqlite::Result<()> {

    try!(ensure_version_table(conn));

    let all_versions = match get_migration_versions(migrate_directory) {
        Ok(v) => v,
        Err(e) => return Err(rusqlite::Error::InvalidParameterName(e.to_string()))
    };

    let installed_versions = try!(get_existing_versions(conn));

    let mut missing_versions = vec![];

    for v in all_versions {
        match installed_versions.contains(&v) {
            true => {},
            false => missing_versions.push(v)
        }
    }

    for v in missing_versions {
        let sql = match get_file_contents(&v.path) {
            Ok(s) => s,
            Err(e) => return Err(rusqlite::Error::InvalidParameterName(e.to_string()))
        };

        try!(conn.execute_batch(&sql));
        try!(conn.execute("INSERT INTO schema_migrations (version) VALUES ($1)", &[&(v.version as i64)]));
    }

    Ok(())
}

fn get_existing_versions(conn: &Connection) -> rusqlite::Result<Vec<Version>> {
    let mut stmt = try!(conn.prepare("SELECT version FROM schema_migrations"));
    let version_itr = try!(stmt.query_map(&[], |row| {
        Version {
            path: "".to_string(),
            version: row.get::<i32, i64>(0) as u32
        }
    }));

    let mut versions = vec![];

    for v in version_itr {
        versions.push(try!(v));
    }

    versions.sort();

    Ok(versions)
}

fn get_file_contents(path: &str) -> io::Result<String> {
    let mut f = try!(File::open(&path));
    let mut contents = String::new();
    try!(f.read_to_string(&mut contents));
    Ok(contents)
}

fn get_migration_versions(migrations_dir: &str) -> io::Result<Vec<Version>> {

    let all_paths = try!(fs::read_dir(migrations_dir));


    let sql_files = all_paths.filter_map(|dirent| dirent.ok())
                             .map(|dirent| dirent.path())
                             .filter(|path| {
                                 match path.extension() {
                                     None => false,
                                     Some(s) => s == "sql",
                                 }
                             });

     let mut res = vec![];
     for file in sql_files {
         let version = try!(calculate_version(&file));
         let path = match file.to_str() {
             Some(p) => p,
             None => return Err(io::Error::new(io::ErrorKind::Other, "Cant read file")),
         };
         res.push(Version {
             path: path.to_string(),
             version: version,
         });
     }
     res.sort();
     Ok(res)
}

fn calculate_version(path: &Path) -> io::Result<u32> {
    let file_name = match path.file_stem() {
        Some(s) => s,
        None => return Err(io::Error::new(io::ErrorKind::Other, "Could not determine filename")),
    };

    match file_name.to_str() {
        None => Err(io::Error::new(io::ErrorKind::Other, "Could not get string path")),
        Some(s) => {
            let mut parts = s.split("__");
            let version = match parts.next() {
                Some(s) => s,
                None => return Err(io::Error::new(io::ErrorKind::Other, "Invalid filename")),
            };

            let version = match version.parse::<u32>() {
                Ok(v) => v,
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Invalid filename: {}", e)))
            };

            Ok(version)
        }
    }
}

const CREATE_VERSION_TABLE: &'static str = "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER NOT NULL)";

fn ensure_version_table(conn: &Connection) -> rusqlite::Result<()> {
    try!(conn.execute(CREATE_VERSION_TABLE, &[]));
    Ok(())
}

#[test]
fn test_calculate_version() {
    assert_eq!(1u32, calculate_version(&Path::new("some/thing/001__hi_there.sql")).unwrap());
    assert_eq!(45u32, calculate_version(&Path::new("some/thing/045__hi_there.sql")).unwrap());
    assert_eq!(101u32, calculate_version(&Path::new("some/thing/101__hi_there.sql")).unwrap());
}
