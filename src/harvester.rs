extern crate pibq;

use std::thread;
use std::time::Duration;

use pibq::bluetherm;

fn main() {
    let c = bluetherm::Connection::new("/Users/delbert/Development/pi-b-q-rust/pipe", Some(1000)).unwrap();
    println!("It's harvesting time!");

    thread::sleep(Duration::from_secs(5));

    match c.get_events() {
        Ok(d) => {
            for evt in &d {
                println!("{}", evt);
            }
        },
        _ => {}
    }

    println!("whew.  going to wait, then wait");
    thread::sleep(Duration::from_secs(5));

    for _ in 0 .. 3 {
        match c.wait() {
            Ok(evt) => println!("{}", evt),
            _ => {}
        }
    }

    println!("threads: {}", c.is_ok());
    drop(c);
    println!("did I get here?");

    // loop {
    //     thread::sleep(Duration::from_secs(5));
    // }
}
