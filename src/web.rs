extern crate pibq;

use pibq::bluetherm;

fn main() {
    println!("It's webserver time");
    println!("Have a packet: {}", bluetherm::Packet::new());
}
