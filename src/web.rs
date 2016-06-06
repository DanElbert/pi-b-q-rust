extern crate iron;
extern crate router;
extern crate staticfile;
extern crate mount;
extern crate handlebars_iron;
extern crate rustc_serialize;

extern crate getopts;
extern crate rusqlite;

extern crate pibq;

use iron::status;
use iron::prelude::*;
use mount::Mount;
use router::Router;
use staticfile::Static;
use handlebars_iron::{Template, HandlebarsEngine, DirectorySource};

use rustc_serialize::json::{self, ToJson, Json};

use std::collections::BTreeMap;
use std::path::Path;

use pibq::sql;
use pibq::models::{ConnectionStatus, Reading};

struct Bean {
    str: String
}

impl ToJson for Bean {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("str".to_string(), self.str.to_json());
        m.to_json()
    }
}

fn main() {
    let mut router = Router::new();
    router.get("/", handler);

    let mut mount = Mount::new();
    mount
        .mount("/", router)
        .mount("/assets/", Static::new(Path::new("web/assets")));

    let mut template_engine = HandlebarsEngine::new();
    template_engine.add(Box::new(DirectorySource::new("web/templates/", ".hbs")));

    if let Err(r) = template_engine.reload() {
        panic!("{}", r);
    }

    let mut chain = Chain::new(mount);
    chain.link_after(template_engine);

    Iron::new(chain).http("0.0.0.0:3000").unwrap();

    fn handler(_: &mut Request) -> IronResult<Response> {
        let mut resp = Response::new();
        let data = Bean { str: "hello world".to_string() };
        resp.set_mut(Template::new("projects", data)).set_mut(status::Ok);
        Ok(resp)
        //Ok(Response::with((status::Ok, "OK")))
    }
}
