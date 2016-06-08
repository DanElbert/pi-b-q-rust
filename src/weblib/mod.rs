use iron::typemap::Key;
use pibq::sql::pool;

pub mod view_models;
pub mod web_handlers;

pub struct AppDb;
impl Key for AppDb { type Value = pool::SqlitePool; }
