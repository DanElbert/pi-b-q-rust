extern crate pibq;

use std::thread;
use std::time::Duration;

use pibq::bluetherm;

fn main() {
    let c = bluetherm::Connection::new("/Users/delbert/Development/pi-b-q-rust/pipe");
    println!("It's harvesting time!");

    loop {
        thread::sleep(Duration::from_secs(5));
    }
}
