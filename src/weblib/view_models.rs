use rustc_serialize::json::{Json, ToJson};
use std::collections::BTreeMap;

use pibq::models;

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

pub struct ProjectEdit {
    pub title: String,
    pub project: models::Project
}

impl ProjectEdit {
    pub fn new(title: &str, project: Option<models::Project>) -> ProjectEdit {
        let project = match project {
            Some(p) => p,
            None => models::Project::default()
        };

        ProjectEdit {
            title: title.to_string(),
            project: project
        }
    }
}

impl ToJson for ProjectEdit {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("title".to_string(), self.title.to_json());
        m.insert("project".to_string(), self.project.to_json());
        m.to_json()
    }
}
