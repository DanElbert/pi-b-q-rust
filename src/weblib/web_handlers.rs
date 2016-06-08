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
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::Read;
use url;


use pibq::sql;
use pibq::sql::pool::{SqlitePooledConnection};
use pibq::models::{Project};
use super::view_models;
use super::AppDb;

#[derive(Clone, Debug)]
struct WebError {
    msg: String
}

impl WebError {
    fn new(msg: &str) -> WebError {
        WebError {
            msg: msg.to_string()
        }
    }
}

impl Error for WebError {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for WebError {
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

fn get_project_from_route(request: &mut Request, conn: &rusqlite::Connection) -> IronResult<Project> {
    let mut project = None;
    match request.extensions.get::<Router>().unwrap().find("id") {
        Some(id) => {
            match id.parse::<i64>() {
                Err(e) => return Err(IronError::new(e, status::InternalServerError)),
                Ok(id) => {
                    project = match sql::get_project(&conn, id) {
                        Err(e) => return Err(IronError::new(e, status::InternalServerError)),
                        Ok(p) => { p }
                    }
                }
            }
        },
        _ => {}
    }

    match project {
        Some(p) => {
            Ok(p)
        },
        None => {
            Err(IronError::new(WebError::new("not found"), status::NotFound))
        }
    }
}

fn assign_project_fields(project: &mut Project, data: &mut HashMap<String, String>, errors: &mut Vec<String>) {
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

    assign_project_fields(&mut project, &mut data, &mut errors);

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
    let conn = try!(get_connection(request));
    let project = try!(get_project_from_route(request, &conn));

    let model = view_models::ProjectEdit::new("Project", Some(project), vec![]);
    render_template("show_project", model)
}

pub fn edit_project(request: &mut Request) -> IronResult<Response> {
    let conn = try!(get_connection(request));
    let project = try!(get_project_from_route(request, &conn));

    let model = view_models::ProjectEdit::new("Edit Project", Some(project), vec![]);
    render_template("edit_project", model)
}

pub fn update_project(request: &mut Request) -> IronResult<Response> {
    let mut data = try!(parse_body(request));
    let conn = try!(get_connection(request));
    let mut project = try!(get_project_from_route(request, &conn));

    let mut errors: Vec<String> = vec![];

    assign_project_fields(&mut project, &mut data, &mut errors);

    if errors.len() > 0 {
        let model = view_models::ProjectEdit::new("Edit Project", Some(project), errors);
        return render_template("edit_project", model);
    } else {
        project.updated_at = Local::now();
        try!(db_unwrap(sql::update_project(&conn, &mut project)));
        return redirect("/");
    }
}

pub fn project_data(request: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();

    let conn = try!(get_connection(request));
    let project = try!(get_project_from_route(request, &conn));

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

    let readings = try!(db_unwrap(sql::get_project_readings(&conn, &project, after)));
    let status = try!(db_unwrap(sql::get_latest_connection_status(&conn)));

    let connected = match status {
        Some(s) => s.is_connect,
        None => false
    };

    let model = view_models::ProjectReadings::new(project, connected, readings);
    let jsonstr = match rustc_serialize::json::encode(&model.to_json()) {
        Err(e) => return Err(IronError::new(e, status::InternalServerError)),
        Ok(str) => str
    };

    resp.set_mut(jsonstr).set_mut(status::Ok);
    Ok(resp)
}
