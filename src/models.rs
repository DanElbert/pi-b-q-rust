use chrono::datetime::DateTime;
use chrono::offset::local::Local;
use rusqlite;

pub struct ConnectionStatus {
    pub id: i64,
    pub is_connect: bool,
    pub is_disconnect: bool,
    pub info: Option<String>,
    pub created_at: DateTime<Local>
}

impl ConnectionStatus {
    pub fn new() -> ConnectionStatus {
        ConnectionStatus {
            id: 0,
            is_connect: false,
            is_disconnect: false,
            info: None,
            created_at: Local::now()
        }
    }
}
