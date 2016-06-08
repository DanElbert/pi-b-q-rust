use rustc_serialize::json::{Json, ToJson};
use std::collections::BTreeMap;

use pibq::models::{self, DbObject};

#[derive(RustcEncodable, RustcDecodable)]
pub struct ProjectIndex {
    pub projects: Vec<models::Project>,
    pub title: String
}

impl ProjectIndex {
    pub fn new(title: &str, projects: Vec<models::Project>) -> Self {
        ProjectIndex {
            projects: projects,
            title: title.to_string()
        }
    }
}

impl ToJson for ProjectIndex {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("title".to_string(), self.title.to_json());
        m.insert("projects".to_string(), self.projects.to_json());
        m.to_json()
    }
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct ProjectEdit {
    pub title: String,
    pub project: models::Project,
    pub errors: Vec<String>
}

impl ProjectEdit {
    pub fn new(title: &str, project: Option<models::Project>, errors: Vec<String>) -> ProjectEdit {
        let project = match project {
            Some(p) => p,
            None => models::Project::default()
        };

        ProjectEdit {
            title: title.to_string(),
            project: project,
            errors: errors
        }
    }

    pub fn is_new(&self) -> bool {
        self.project.is_new()
    }

    pub fn has_errors(&self) -> bool {
        self.errors.len() > 0
    }
}

impl ToJson for ProjectEdit {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("title".to_string(), self.title.to_json());
        m.insert("project".to_string(), self.project.to_json());
        m.insert("errors".to_string(), self.errors.to_json());
        m.insert("is_new".to_string(), self.is_new().to_json());
        m.insert("has_errors".to_string(), self.has_errors().to_json());
        m.to_json()
    }
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct ProjectReadings {
    project: models::Project,
    connected: bool,
    readings: Vec<models::Reading>
}

impl ProjectReadings {
    pub fn new(project: models::Project, connected: bool, readings: Vec<models::Reading>) -> Self {
        ProjectReadings {
            project: project,
            connected: connected,
            readings: readings
        }
    }
}

impl ToJson for ProjectReadings {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("connected".to_string(), self.connected.to_json());
        m.insert("readings".to_string(), self.readings.to_json());
        m.to_json()
    }
}
