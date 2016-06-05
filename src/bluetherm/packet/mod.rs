
mod crc;
mod converters;
pub mod data_flags;
pub mod message_type;

use std::cmp;
use std::fmt;

pub struct Packet {
  pub data: [u8; 128]
}

impl Packet {

  pub fn new() -> Packet {
    Packet { data: [0u8; 128] }
  }

  pub fn default() -> Packet {
      let mut p = Packet::new();
      p.set_command_id(message_type::RETRIEVE_INFO);
      p.set_version(1);
      p.set_data_flags(data_flags::DEFAULT);
      p.apply_checksum();
      p
  }

  pub fn from_bytes(bytes: &[u8]) -> Packet {
      let mut data = [0u8; 128];
      for x in 0 .. (cmp::min(data.len(), bytes.len())) {
          data[x] = bytes[x];
      }
      Packet { data: data }
  }

  pub fn temp_packet() -> Packet {
      let mut p = Packet::new();
      p.set_command_id(message_type::RETRIEVE_INFO);
      p.set_version(1);
      p.set_data_flags(data_flags::TEMPS);
      p.apply_checksum();
      p
  }

  pub fn calculate_checksum(&self) -> u16 {
      crc::compute_checksum(&self.data[0 .. 126])
  }

  pub fn apply_checksum(&mut self) {
      let crc = self.calculate_checksum();
      self.set_checksum(crc);
  }

  pub fn is_checksum_valid(&self) -> bool {
      self.get_checksum() == self.calculate_checksum()
  }

  pub fn get_command_id(&self) -> message_type::MessageType {
    self.get_field(0, 1, converters::message_type_out)
  }

  pub fn set_command_id(&mut self, value: message_type::MessageType) {
    self.set_field(0, 1, value, converters::message_type_in);
  }

  pub fn get_version(&self) -> u8 {
    self.get_field(1, 1, converters::noop_out)
  }

  pub fn set_version(&mut self, value: u8) {
    self.set_field(1, 1, value, converters::noop_in);
  }

  pub fn get_data_flags(&self) -> data_flags::DataFlags {
    self.get_field(2, 2, converters::data_flags_out)
  }

  pub fn set_data_flags(&mut self, value: data_flags::DataFlags) {
    self.set_field(2, 2, value, converters::data_flags_in);
  }

  pub fn get_serial_number(&self) -> String {
    self.get_field(4, 10, converters::string_out)
  }

  pub fn set_serial_number(&mut self, value: &str) {
    self.set_field(4, 10, value, converters::string_in);
  }

  pub fn get_sensor1_reading(&self) -> Option<f64> {
      self.get_field(54, 4, converters::temperature_out)
  }

  pub fn set_sensor1_reading(&mut self, value: Option<f64>) {
      self.set_field(54, 4, value, converters::temperature_in);
  }

  pub fn get_sensor2_reading(&self) -> Option<f64> {
      self.get_field(74, 4, converters::temperature_out)
  }

  pub fn set_sensor2_reading(&mut self, value: Option<f64>) {
      self.set_field(74, 4, value, converters::temperature_in);
  }

  pub fn get_battery_volts(&self) -> f32 {
      self.get_field(0x5E, 2, converters::battery_out)
  }

  pub fn set_battery_volts(&mut self, value: f32) {
      self.set_field(0x5E, 2, value, converters::battery_in);
  }

  pub fn get_checksum(&self) -> u16 {
      self.get_field(0x7E, 2, converters::word_out)
  }

  pub fn set_checksum(&mut self, value: u16) {
      self.set_field(0x7E, 2, value, converters::word_in);
  }

  fn get_field<T>(&self, start: usize, length: usize, converter: fn(&[u8]) -> T) -> T {
    let chunk = &self.data[start .. (start + length)];
    converter(chunk)
  }

  fn set_field<T>(&mut self, start: usize, length: usize, value: T, converter: fn(T, &mut [u8])) {
    let chunk = &mut self.data[start .. (start + length)];
    converter(value, chunk);
  }

}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Packet<"));

        for i in 0 .. self.data.len() {
            if i != 0 { try!(write!(f, ":")); }
            try!(write!(f, "{}", self.data[i]));
        }

        write!(f, ">")
    }
}
