use chrono::datetime::DateTime;
use chrono::offset::local::Local;
use handlebars_iron::Template;
use iron;
use iron::modifiers::Redirect;
use iron::prelude::*;
use iron::status;
use persistent::{self, PersistentError};
use router::Router;
use rusqlite;
use rustc_serialize::json::{ToJson};
use std::cmp;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash;
use std::io::Read;
use url;


use pibq::sql;
use pibq::sql::pool::{SqlitePooledConnection};
use pibq::models::{ConnectionStatus, Project, Reading};
use super::view_models;
use super::AppDb;

trait DefaultValue<K, V> {
    fn remove_or_default(&mut self, key: &K, default: V) -> V;
}

impl<K, V> DefaultValue<K, V> for HashMap<K, V> where K: cmp::Eq, K: hash::Hash {
    fn remove_or_default(&mut self, key: &K, default: V) -> V {
        match self.remove(key) {
            None => default,
            Some(v) => v
        }
    }
}

#[derive(Clone, Debug)]
pub struct UrlParseError {
    msg: String
}

impl UrlParseError {
    fn new(msg: &str) -> Self {
        UrlParseError {
            msg: msg.to_string()
        }
    }
}

impl Error for UrlParseError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for UrlParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}


fn get_connection(request: &mut Request) -> IronResult<SqlitePooledConnection> {
    let pool = match request.get::<persistent::Read<AppDb>>() {
        Err(e) => return Err(IronError::new(e, status::InternalServerError)),
        Ok(p) => p
    };

    match pool.get() {
        Err(e) => return Err(IronError::new(e, status::InternalServerError)),
        Ok(c) => Ok(c)
    }
}

fn db_unwrap<T>(result: rusqlite::Result<T>) -> IronResult<T> {
    match result {
        Err(e) => Err(IronError::new(e, status::InternalServerError)),
        Ok(r) => Ok(r)
    }
}

fn render_template<T>(template: &str, model: T) -> IronResult<Response> where T: ToJson {
    Ok(Response::with(status::Ok).set(Template::new(template, model)))
}

fn redirect(url: &str) -> IronResult<Response> {
    let url = match iron::Url::parse(url) {
        Err(e) => return Err(IronError::new(UrlParseError::new(&e), status::InternalServerError)),
        Ok(u) => u
    };

    Ok(Response::with(Redirect(url)))
}

fn parse_body(request: &mut Request) -> IronResult<HashMap<String, String>> {
    let mut body = vec![];
    match request.body.read_to_end(&mut body) {
        Err(e) => return Err(IronError::new(e, status::InternalServerError)),
        Ok(_) => {}
    };

    let mut map = HashMap::new();

    for pair in url::form_urlencoded::parse(&body).into_owned() {
        let (name, value) = pair;
        map.insert(name, value);
    }

    Ok(map)
}

pub fn projects_index(request: &mut Request) -> IronResult<Response> {
    let conn = try!(get_connection(request));
    let projects = try!(db_unwrap(sql::get_projects(&conn)));
    let model = view_models::ProjectIndex::new("Projects", projects);

    render_template("projects", model)
}

pub fn new_project(request: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();
    let model = view_models::ProjectEdit::new("Create Project", None);

    render_template("edit_project", model)
}

pub fn create_project(request: &mut Request) -> IronResult<Response> {
    let mut data = try!(parse_body(request));
    let mut project = Project::default();

    project.name = data.remove_or_default(&"name".to_string(), "DEFAULT".to_string()).to_string();
    //project.start = data.remove_or_default("start", "");

    // for pair in url::form_urlencoded::parse(&body).into_owned() {
    //     let (name, value) = pair;
    //
    //     match name.as_str() {
    //         "name" => project.name = value,
    //         "start" => project.start = value.parse::<DateTime<Local>>().unwrap(),
    //         "end" => project.start = value.parse::<DateTime<Local>>().unwrap(),
    //         "sensor1_name" => project.sensor1_name = value,
    //         "sensor2_name" => project.sensor2_name = value,
    //         _ => {}
    //     }
    // }

    // let pool = request.get::<persistent::Read<AppDb>>().unwrap();
    // let conn: SqlitePooledConnection = pool.get().unwrap();
    //
    // sql::insert_project(&conn, &mut project);

    redirect("/")
}

pub fn show_project(request: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();
    let mut project: Option<Project> = None;

    let pool = request.get::<persistent::Read<AppDb>>().unwrap();
    let conn: SqlitePooledConnection = pool.get().unwrap();

    match request.extensions.get::<Router>().unwrap().find("id") {
        Some(id) => {
            let id = id.parse::<i64>().unwrap();
            project = match sql::get_project(&conn, id) {
                Err(e) => return Err(IronError::new(e, status::InternalServerError)),
                Ok(p) => { p }
            }
        },
        _ => {}
    }

    match project {
        Some(p) => {
            let model = view_models::ProjectEdit::new("Project", Some(p));
            resp.set_mut(Template::new("show_project", model)).set_mut(status::Ok);
        },
        None => {
            resp.set_mut(status::NotFound);
        }
    }

    Ok(resp)
}

pub fn edit_project(request: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();
    Ok(resp)
}

pub fn update_project(request: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();
    Ok(resp)
}
