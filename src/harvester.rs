extern crate getopts;
extern crate rusqlite;
extern crate pibq;

use std::env;
use std::time::{Duration, Instant};
use getopts::Options;

use pibq::bluetherm;
use pibq::sql;
use pibq::models::{ConnectionStatus, Reading};

// interval between sending query packtets, in ms
const QUERY_INTERVAL: u64 = 3000;

// if no packets are returned, amount of time to wait before creating a timeout error, in ms
const TIMEOUT_INTERVAL: u64 = 6500;

const HEARTBEAT_INTERVAL: u64 = 1000;

struct Harvester {
    sql_conn: rusqlite::Connection,
    bt_conn: Option<bluetherm::Connection>,
    serial: String,
    disconnected: bool,
    disconnect_reason: Option<bluetherm::ConnectionEvent>,
    send_interval: Duration,
    timeout_interval: Duration,
    last_send: Option<Instant>,
    last_receive: Option<Instant>
}

impl Harvester {
    fn new(sql_conn: rusqlite::Connection, serial: &str) -> Harvester {
        Harvester {
            sql_conn: sql_conn,
            bt_conn: Some(Harvester::connect_bluetherm(serial)),
            serial: serial.to_string(),
            disconnected: true,
            disconnect_reason: None,
            send_interval: Duration::from_millis(QUERY_INTERVAL),
            timeout_interval: Duration::from_millis(TIMEOUT_INTERVAL),
            last_send: None,
            last_receive: None
        }
    }

    fn connect_bluetherm(serial: &str) -> bluetherm::Connection {
        bluetherm::Connection::new(serial, Some(HEARTBEAT_INTERVAL)).unwrap()
    }

    fn start(&mut self) {
        loop {
            match self.last_send {
                Some(i) if i.elapsed() < self.send_interval => {},
                _ => {
                    self.send_packet();
                    self.last_send = Some(Instant::now());
                }
            }

            let event = self.get_bt_conn().wait().unwrap();

            println!("{}", event);
            match event {
                bluetherm::ConnectionEvent::Packet(p) => {
                    self.record_packet(p);
                    self.last_receive = Some(Instant::now());
                },
                e @ bluetherm::ConnectionEvent::InvalidPacket(_) => { self.bt_error(e); },
                e @ bluetherm::ConnectionEvent::ReadError(_) => { self.bt_error(e); },
                e @ bluetherm::ConnectionEvent::BadData(_) => { self.bt_error(e); },
                e @ bluetherm::ConnectionEvent::Heartbeat => {
                    match self.last_receive {
                        None => {
                            match self.last_send {
                                None => {},
                                Some(sent) => {
                                    if sent.elapsed() > self.timeout_interval {
                                        self.bt_error(e);
                                    }
                                }
                            }
                        },
                        Some(rec) => {
                            if rec.elapsed() > self.timeout_interval {
                                self.bt_error(e);
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_bt_conn(&mut self) -> &mut bluetherm::Connection {
        self.bt_conn.as_mut().unwrap()
    }

    fn record_packet(&mut self, packet: bluetherm::Packet) {
        let mut reading = Reading::new();
        reading.value1 = packet.get_sensor1_reading();
        reading.value2 = packet.get_sensor2_reading();
        sql::insert_reading(&self.sql_conn, &mut reading).unwrap();
    }

    fn send_packet(&mut self) {
        let p = bluetherm::Packet::temp_packet();
        self.get_bt_conn().send(&p).unwrap();
    }

    fn bt_error(&mut self, evt: bluetherm::ConnectionEvent) {
        let mut report_error = true;

        if self.disconnected {
            match &self.disconnect_reason {
                &None => {},
                &Some(ref reason) => {
                    match (reason, &evt) {
                        (&bluetherm::ConnectionEvent::InvalidPacket(_), &bluetherm::ConnectionEvent::InvalidPacket(_)) |
                        (&bluetherm::ConnectionEvent::ReadError(_), &bluetherm::ConnectionEvent::ReadError(_)) |
                        (&bluetherm::ConnectionEvent::BadData(_), &bluetherm::ConnectionEvent::BadData(_)) |
                        (&bluetherm::ConnectionEvent::Heartbeat, &bluetherm::ConnectionEvent::Heartbeat) => {
                            report_error = false;
                        },
                        _ => {},
                    }
                }
            }
        }

        if report_error {
            let msg = match evt {
                bluetherm::ConnectionEvent::InvalidPacket(p) => { format!("Invalid Packet: [{}]", p) },
                bluetherm::ConnectionEvent::ReadError(err) => { format!("Read Error: {}", err) },
                bluetherm::ConnectionEvent::BadData(_) => { format!("Bad Data Read") },
                bluetherm::ConnectionEvent::Heartbeat => { "Timeout".to_string() },
                _ => "Unknown Error".to_string()
            };

            let mut s = ConnectionStatus::new();
            s.is_disconnect = true;
            s.info = Some(msg);
            sql::insert_connection_status(&self.sql_conn, &mut s).unwrap();
        }

        self.disconnected = true;

        println!("error, killing old connection");

        let old = self.bt_conn.take();
        drop(old.unwrap());

        println!("old conneciton dropped; making new");

        self.bt_conn = Some(Harvester::connect_bluetherm(&self.serial));

        println!("new connection made");
    }
}

// fn harvest(sql_conn: rusqlite::Connection, serial: &str) {
//
//     loop {
//         // if disconnected {
//         //     bt_conn = bluetherm::Connection::new(serial, Some(1000)).unwrap();
//         //     let mut s = ConnectionStatus::new();
//         //     s.is_disconnect = true;
//         //     sql::insert_connection_status(&sql_conn, &mut s).unwrap();
//         // }
//
//
//     }
//
// }

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

    let mut h = Harvester::new(db, &serial);
    h.start();
}
