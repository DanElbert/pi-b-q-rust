extern crate pi_b_q_rust;
use pi_b_q_rust::bluetherm;

#[test]
fn test_command_id() {
    let mut p = bluetherm::Packet::new();
    p.set_command_id(12);
    assert_eq!(12, p.get_command_id());
}

#[test]
fn test_data_flags() {
    let mut p = bluetherm::Packet::new();
    p.set_data_flags(bluetherm::data_flags::SENSOR_1_TEMPERATURE | bluetherm::data_flags::SENSOR_2_TEMPERATURE);
    assert_eq!(bluetherm::data_flags::SENSOR_1_TEMPERATURE | bluetherm::data_flags::SENSOR_2_TEMPERATURE, p.get_data_flags());
}

#[test]
fn test_serial_number() {
    let mut p = bluetherm::Packet::new();
    p.set_serial_number("abcdefc");
    assert_eq!("abcdefc", p.get_serial_number());
}
