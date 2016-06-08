use chrono::datetime::DateTime;
use chrono::offset::local::Local;
use chrono::offset::TimeZone;
use rustc_serialize::Encodable;
use rustc_serialize::json::{self, Json, ToJson};
use std::collections::BTreeMap;

pub trait DbObject {
    fn get_id(&self) -> i64;

    fn is_new(&self) -> bool {
        self.get_id() == 0
    }

    fn is_persisten(&self) -> bool {
        !self.is_new()
    }
}

pub trait CustomToJson {
    fn to_json(&self) -> Json;
}

impl<Tz> CustomToJson for DateTime<Tz>
    where Tz: TimeZone+Encodable, Tz::Offset: Encodable {
    fn to_json(&self) -> Json {
        json::encode(self).unwrap().to_json()
    }
}

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

impl DbObject for ConnectionStatus {
    fn get_id(&self) -> i64 {
        self.id
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

    pub fn default() -> Project {
        Self::new("".to_string(), Local::now(), Local::now(), "".to_string(), "".to_string())
    }
}

impl DbObject for Project {
    fn get_id(&self) -> i64 {
        self.id
    }
}

impl ToJson for Project {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("id".to_string(), self.id.to_json());
        m.insert("name".to_string(), self.name.to_json());
        m.insert("start".to_string(), self.start.to_json());
        m.insert("end".to_string(), self.end.to_json());
        m.insert("sensor1_name".to_string(), self.sensor1_name.to_json());
        m.insert("sensor2_name".to_string(), self.sensor2_name.to_json());
        m.insert("created_at".to_string(), self.created_at.to_json());
        m.insert("updated_at".to_string(), self.updated_at.to_json());

        m.to_json()
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

impl DbObject for Reading {
    fn get_id(&self) -> i64 {
        self.id
    }
}
