extern crate pi_b_q_rust;

use pi_b_q_rust::bluetherm;

fn main() {

    let mut p = bluetherm::Packet::new();
    let mut cid = p.get_command_id();
    println!("{}", cid);

    p.set_command_id(12);
    cid = p.get_command_id();
    println!("{}", cid);

    p.set_data_flags(bluetherm::data_flags::SENSOR_1_TEMPERATURE | bluetherm::data_flags::SENSOR_2_TEMPERATURE);
    println!("{}", p.get_data_flags());

    p.set_serial_number("abcdefc");
        println!("{}", p.get_serial_number());

    println!("Hello, world!");
}
