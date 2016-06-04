extern crate getopts;
extern crate rusqlite;
extern crate pibq;

use std::env;
use std::thread;
use std::time::Duration;
use getopts::Options;

use pibq::bluetherm;
use pibq::sql;
use pibq::models::{ConnectionStatus};

fn harvest(sql_conn: rusqlite::Connection, serial: &str) {
    let mut bt_conn: Option<bluetherm::Connection> = None;

    loop {
        if bt_conn.is_none() {
            bt_conn = Some(bluetherm::Connection::new(serial, Some(1000)).unwrap());
            let mut s = ConnectionStatus::new();
            s.is_disconnect = true;
            sql::insert_connection_status(&sql_conn, &mut s).unwrap();
        }




        thread::sleep(Duration::from_secs(3));
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
    opts.reqopt("s", "serial", "tty serial device", "DEV");
    opts.optopt("d", "dbfile", "sqlite DB file", "FILE");
    opts.optopt("m", "migrations", "migration folder", "DIR");
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
    let serial = matches.opt_str("s").unwrap();
    let dbfile = matches.opt_str("d").unwrap_or("pibq.sqlite".to_string());
    let migrations = matches.opt_str("m").unwrap_or("migrations".to_string());

    let db = sql::get_connection(&dbfile, Some(migrations)).unwrap();

    harvest(db, &serial);


    // let c = bluetherm::Connection::new(&serial, Some(1000)).unwrap();
    // println!("It's harvesting time!");
    //
    // thread::sleep(Duration::from_secs(5));
    //
    // match c.get_events() {
    //     Ok(d) => {
    //         for evt in &d {
    //             println!("{}", evt);
    //         }
    //     },
    //     _ => {}
    // }
    //
    // println!("whew.  going to wait, then wait");
    // thread::sleep(Duration::from_secs(5));
    //
    // for _ in 0 .. 3 {
    //     match c.wait() {
    //         Ok(evt) => println!("{}", evt),
    //         _ => {}
    //     }
    // }
    //
    // println!("threads: {}", c.is_ok());
    // drop(c);
    // println!("did I get here?");

    // loop {
    //     thread::sleep(Duration::from_secs(5));
    // }
}
