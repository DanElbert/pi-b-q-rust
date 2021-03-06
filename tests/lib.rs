extern crate pibq;
use pibq::bluetherm;

#[test]
fn test_command_id() {
    let mut p = bluetherm::Packet::new();
    p.set_command_id(bluetherm::message_type::RETRIEVE_INFO);
    assert_eq!(bluetherm::message_type::RETRIEVE_INFO, p.get_command_id());
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

#[test]
fn test_temperatures() {
    let mut p = bluetherm::Packet::new();
    p.set_sensor1_reading(Some(32.0f64));
    p.set_sensor2_reading(Some(100.5f64));

    assert_eq!(Some(32.0f64), p.get_sensor1_reading());
    assert_eq!(Some(100.5f64), p.get_sensor2_reading());

    p.set_sensor1_reading(None);
    assert_eq!(None, p.get_sensor1_reading());
}

#[test]
fn test_example_packet_crc() {
    let mut data = [0u8; 128];
    data[0] = 1;
    data[1] = 1;
    data[2] = 0xFF;
    data[3] = 0xFF;
    data[126] = 0xA5;
    data[127] = 0x88;
    let p = bluetherm::Packet { data: data };
    assert!(p.is_checksum_valid());
}
