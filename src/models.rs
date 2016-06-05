use chrono::datetime::DateTime;
use chrono::offset::local::Local;

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

pub struct Reading {
    pub id: i64,
    pub value1: Option<f64>,
    pub value2: Option<f64>,
    pub timestamp: DateTime<Local>
}

impl Reading {
    pub fn new() -> Reading {
        Reading {
            id: 0,
            value1: None,
            value2: None,
            timestamp: Local::now()
        }
    }
}
