extern crate chrono;
extern crate getopts;
extern crate handlebars_iron;
extern crate iron;
extern crate logger;
extern crate mount;
extern crate persistent;
extern crate r2d2;
extern crate router;
extern crate rusqlite;
extern crate rustc_serialize;
extern crate staticfile;
extern crate url;

extern crate pibq;

mod weblib;

use getopts::Options;
use handlebars_iron::{HandlebarsEngine, DirectorySource};
use iron::{AfterMiddleware};
use iron::prelude::*;
use mount::Mount;
use router::Router;
use staticfile::Static;
use std::env;
use std::path::Path;


use pibq::sql;
use pibq::sql::pool::{SqlitePool};
use weblib::AppDb;
use weblib::web_handlers;

struct ErrorHandler;

impl AfterMiddleware for ErrorHandler {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        println!("Error Encountered: {:?}", err.error);
        Ok(err.response)
    }
}

struct WebServer {
    sql_pool: SqlitePool,
    asset_path: String,
    template_path: String,
    port: String
}

impl WebServer {
    pub fn new(sql_pool: SqlitePool, web_root: &str, port: &str) -> Self {
        WebServer {
            sql_pool: sql_pool,
            asset_path: web_root.to_string() + "/assets/",
            template_path: web_root.to_string() + "/templates/",
            port: port.to_string()
        }
    }

    pub fn start(&mut self) {
        let mut router = Router::new();
        router.get("/", |request: &mut Request| { web_handlers::projects_index(request) }, "index");
        router.get("/projects/new", |request: &mut Request| { web_handlers::new_project(request) }, "new_project");
        router.post("/projects/new", |request: &mut Request| { web_handlers::create_project(request) }, "create_project");
        router.get("/projects/:id", |request: &mut Request| { web_handlers::show_project(request) }, "project");
        router.get("/projects/:id/edit", |request: &mut Request| { web_handlers::edit_project(request) }, "edit_project");
        router.post("/projects/:id", |request: &mut Request| { web_handlers::update_project(request) }, "update_project");
        router.get("/projects/:id/data.json", |request: &mut Request| { web_handlers::project_data(request) }, "project_data");

        let mut mount = Mount::new();
        mount
            .mount("/", router)
            .mount("/assets/", Static::new(Path::new(&self.asset_path)).cache(std::time::Duration::from_secs(2 * 60 * 60)));

        let mut template_engine = HandlebarsEngine::new();
        template_engine.add(Box::new(DirectorySource::new(&self.template_path, ".hbs")));

        if let Err(r) = template_engine.reload() {
            panic!("{:?}", r);
        }

        let (logger_before, logger_after) = logger::Logger::new(None);

        let mut chain = Chain::new(mount);
        chain.link(persistent::Read::<AppDb>::both(self.sql_pool.clone()));
        chain.link_after(template_engine);
        chain.link_after(ErrorHandler);
        chain.link_before(logger_before);
        chain.link_after(logger_after);

        let binding = "0.0.0.0:".to_string() + &self.port;

        Iron::new(chain).http(binding.as_str()).unwrap();
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("d", "dbfile", "sqlite DB file", "FILE");
    opts.optopt("w", "webroot", "root of web files", "DIR");
    opts.optopt("p", "port", "port to listen on", "PORT");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            println!("{}", f.to_string());
            print_usage(&program, opts);
            return;
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let dbfile = matches.opt_str("d").unwrap_or("pibq.sqlite".to_string());
    let webroot = matches.opt_str("w").unwrap_or("web".to_string());
    let port = matches.opt_str("p").unwrap_or("3000".to_string());

    let db_pool = sql::get_pool(&dbfile, Some(5));

    let mut w = WebServer::new(db_pool, &webroot, &port);
    w.start();
}
