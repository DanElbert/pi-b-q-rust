
mod converters;
pub mod data_flags;

pub struct Packet {
  pub data: [u8; 128]
}



impl Packet {

  pub fn new() -> Packet {
    Packet { data: [0u8; 128] }
  }

  pub fn get_command_id(&self) -> u8 {
    self.get_field(0, 1, converters::noop_out)
  }

  pub fn set_command_id(&mut self, value: u8) {
    self.set_field(0, 1, value, converters::noop_in);
  }

  pub fn get_version(&self) -> u8 {
    self.get_field(1, 1, converters::noop_out)
  }

  pub fn set_version(&mut self, value: u8) {
    self.set_field(1, 1, value, converters::noop_in);
  }

  pub fn get_data_flags(&self) -> data_flags::DataFlags {
    let word = self.get_field(2, 2, converters::word_out);
    match data_flags::DataFlags::from_bits(word) {
        Some(flags) => flags,
        None => data_flags::NONE
    }
  }

  pub fn set_data_flags(&mut self, value: data_flags::DataFlags) {
    let word = value.raw_bits();
    self.set_field(2, 2, word, converters::word_in);
  }

  pub fn get_serial_number(&self) -> String {
    self.get_field(4, 10, converters::string_out)
  }

  pub fn set_serial_number(&mut self, value: &str) {
    self.set_field(4, 10, value, converters::string_in);
  }

  pub fn get_sensor1_reading(&self) -> f32 {
      self.get_field(54, 4, converters::temperature_out)
  }

  pub fn get_sensor2_reading(&self) -> f32 {
      self.get_field(74, 4, converters::temperature_out)
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
