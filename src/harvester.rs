extern crate pibq;

use std::thread;
use std::time::Duration;

use pibq::bluetherm;

fn main() {
    let c = bluetherm::Connection::new("/home/dan/Development/pi-b-q-rust/pipe").unwrap();
    println!("It's harvesting time!");
    thread::sleep(Duration::from_secs(5));
    println!("threads: {}", c.is_ok());
    drop(c);
    println!("did I get here?");

    // loop {
    //     thread::sleep(Duration::from_secs(5));
    // }
}
