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

pub struct Project {
    pub id: i64,
    pub name: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub sensor1_name: String,
    pub sensor2_name: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>
}

impl Project {
    pub fn new(name: String, start: DateTime<Local>, end: DateTime<Local>, sensor1_name: String, sensor2_name: String) -> Project {
        Project {
            id: 0,
            name: name,
            start: start,
            end: end,
            sensor1_name: sensor1_name,
            sensor2_name: sensor2_name,
            created_at: Local::now(),
            updated_at: Local::now()
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
