extern crate iron;
extern crate router;
extern crate staticfile;
extern crate mount;
extern crate urlencoded;
extern crate handlebars_iron;

extern crate getopts;
extern crate rusqlite;

extern crate pibq;

use iron::status;
use iron::prelude::*;
use mount::Mount;
use router::Router;
use staticfile::Static;

use std::path::Path;


use pibq::sql;
use pibq::models::{ConnectionStatus, Reading};

fn main() {
    let mut router = Router::new();
    router.get("/", handler);

    let mut mount = Mount::new();
    mount
        .mount("/", router)
        .mount("/assets/", Static::new(Path::new("web/assets")));

    Iron::new(mount).http("0.0.0.0:3000").unwrap();

    fn handler(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "OK")))
    }
}
