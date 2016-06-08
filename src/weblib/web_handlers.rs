use chrono::datetime::DateTime;
use chrono::offset::TimeZone;
use chrono::offset::local::Local;
use handlebars_iron::Template;
use iron::headers;
use iron::modifiers::{Header};
use iron::prelude::*;
use iron::status;
use persistent::{self};
use router::Router;
use rusqlite;
use rustc_serialize;
use rustc_serialize::json::{ToJson};
use std::cmp;
use std::collections::HashMap;
use std::hash;
use std::io::Read;
use url;


use pibq::sql;
use pibq::sql::pool::{SqlitePooledConnection};
use pibq::models::{Project};
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

fn parse_date(str: &str) -> IronResult<DateTime<Local>> {
    match Local.datetime_from_str(str, "%Y-%m-%d %H:%M:%S") {
        Err(e) => Err(IronError::new(e, status::InternalServerError)),
        Ok(dt) => Ok(dt)
    }
}

fn render_template<T>(template: &str, model: T) -> IronResult<Response> where T: ToJson {
    Ok(Response::with(status::Ok).set(Template::new(template, model)))
}

fn redirect(url: &str) -> IronResult<Response> {
    Ok(Response::with((status::Found, Header(headers::Location(url.to_string())))))
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

fn parse_query(request: &mut Request) -> IronResult<HashMap<String, String>> {
    let mut map = HashMap::new();

    let url = match url::Url::parse(&format!("{}", request.url)) {
        Err(e) => return Err(IronError::new(e, status::InternalServerError)),
        Ok(u) => u
    };

    for pair in url.query_pairs().into_owned() {
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

pub fn new_project(_: &mut Request) -> IronResult<Response> {
    let model = view_models::ProjectEdit::new("Create Project", None, vec![]);

    render_template("edit_project", model)
}

pub fn create_project(request: &mut Request) -> IronResult<Response> {
    let mut data = try!(parse_body(request));
    let mut project = Project::default();

    let mut errors: Vec<String> = vec![];

    match data.remove("name") {
        Some(ref str) if str.len() > 0 => {
            project.name = str.to_string();
        }
        _ => {
            errors.push("Invalid name".to_string());
        }
    }

    match data.remove("sensor1_name") {
        Some(ref str) if str.len() > 0 => {
            project.sensor1_name = str.to_string();
        }
        _ => {
            errors.push("Invalid sensor1_name".to_string());
        }
    }

    match data.remove("sensor2_name") {
        Some(ref str) if str.len() > 0 => {
            project.sensor2_name = str.to_string();
        }
        _ => {
            errors.push("Invalid sensor2_name".to_string());
        }
    }

    match data.remove("start") {
        None => { errors.push("Invalid start".to_string()); },
        Some(str) => {
            match parse_date(&str) {
                Err(e) => { errors.push(format!("start invalid: {}", e)); },
                Ok(dt) => { project.start = dt; }
            }
        }
    }

    match data.remove("end") {
        None => { errors.push("Invalid end".to_string()); },
        Some(str) => {
            match parse_date(&str) {
                Err(e) => { errors.push(format!("end invalid: {}", e)); },
                Ok(dt) => { project.end = dt; }
            }
        }
    }

    if errors.len() > 0 {
        let model = view_models::ProjectEdit::new("Create Project", Some(project), errors);
        return render_template("edit_project", model);
    } else {
        let conn = try!(get_connection(request));
        try!(db_unwrap(sql::insert_project(&conn, &mut project)));
        return redirect("/");
    }

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
            let model = view_models::ProjectEdit::new("Project", Some(p), vec![]);
            resp.set_mut(Template::new("show_project", model)).set_mut(status::Ok);
        },
        None => {
            resp.set_mut(status::NotFound);
        }
    }

    Ok(resp)
}

pub fn edit_project(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(status::Ok).set("edit project"))
}

pub fn update_project(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(status::Ok).set("update project"))
}

pub fn project_data(request: &mut Request) -> IronResult<Response> {
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
            let mut query = try!(parse_query(request));

            let after = match query.remove("after") {
                None => { None },
                Some(ref str) if str.len() == 0 => { None },
                Some(str) => {
                    match parse_date(&str) {
                        Err(e) => { return Err(e) },
                        Ok(dt) => { Some(dt) }
                    }
                }
            };

            let readings = try!(db_unwrap(sql::get_project_readings(&conn, &p, after)));
            let status = try!(db_unwrap(sql::get_latest_connection_status(&conn)));

            let connected = match status {
                Some(s) => s.is_connect,
                None => false
            };

            let model = view_models::ProjectReadings::new(p, connected, readings);
            let jsonstr = match rustc_serialize::json::encode(&model) {
                Err(e) => return Err(IronError::new(e, status::InternalServerError)),
                Ok(str) => str
            };

            resp.set_mut(jsonstr).set_mut(status::Ok);
        },
        None => {
            resp.set_mut(status::NotFound);
        }
    }

    Ok(resp)
}
