extern crate gcc;

fn main() {
gcc::Config::new()
    .file("vendor/sqlite3/sqlite3.c")
    .compile("libsqlite3.a");
}
