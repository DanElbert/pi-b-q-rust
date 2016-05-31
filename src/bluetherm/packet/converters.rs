use std::str;
use super::data_flags;
use super::message_type;

pub fn noop_out(data: &[u8]) -> u8 {
    data[0]
}

pub fn noop_in(value: u8, buffer: &mut [u8]) {
    buffer[0] = value;
}

pub fn word_out(data: &[u8]) -> u16 {
    let mut value: u16 = data[0] as u16;
    value += (data[1] as u16) << 8;
    value
}

pub fn word_in(value: u16, buffer: &mut [u8]) {
    buffer[0] = (value & 0xFF) as u8;
    buffer[1] = ((value >> 8) & 0xFF) as u8;
}

pub fn string_out(data: &[u8]) -> String {
    let mut value = String::new();
    for x in 0 .. data.len() {
        if data[x] != 0 {
            value.push(data[x] as char);
        }
    }

    value
}

pub fn string_in(value: &str, buffer: &mut [u8]) {
    let bytes = value.as_bytes();
    for x in 0 .. buffer.len() {
        if x < bytes.len() {
            buffer[x] = bytes[x];
        } else {
            buffer[x] = 0;
        }
    }
}

pub fn temperature_out(data: &[u8]) -> f32 {
    let mut int_value: u32 = data[0] as u32;
    int_value += (data[1] as u32) << 8;
    int_value += (data[2] as u32) << 16;
    int_value += (data[3] as u32) << 24;

    let mut value: f32 = 0f32;

    if int_value < 0xFFFFFFFD {
        value = int_value as f32;
        value = (value / 100_000.0f32) - 300.0f32;
    }

    value
}

pub fn temperature_in(value: f32, buffer: &mut [u8]) {
    let int_value: u32 = ((value + 300.0f32) * 100_000.0f32) as u32;
    buffer[0] = (int_value & 0xFF) as u8;
    buffer[1] = ((int_value >> 8) & 0xFF) as u8;
    buffer[2] = ((int_value >> 16) & 0xFF) as u8;
    buffer[3] = ((int_value >> 24) & 0xFF) as u8;
}

pub fn data_flags_out(data: &[u8]) -> data_flags::DataFlags {
    let word = word_out(data);
    match data_flags::DataFlags::from_bits(word) {
        Some(flags) => flags,
        None => data_flags::NONE
    }
}

pub fn data_flags_in(value: data_flags::DataFlags, buffer: &mut [u8]) {
    word_in(value.raw_bits(), buffer);
}

pub fn battery_out(data: &[u8]) -> f32 {
    let word = word_out(data);
    (word as f32) / 1000.0f32
}

pub fn battery_in(value: f32, buffer: &mut [u8]) {
    let word = (value * 1000.0f32) as u16;
    word_in(word, buffer);
}

pub fn message_type_out(data: &[u8]) -> message_type::MessageType {
    let byte = data[0];
    match message_type::MessageType::from_bits(byte) {
        Some(m_type) => m_type,
        None => message_type::NOTHING
    }
}

pub fn message_type_in(value: message_type::MessageType, buffer: &mut [u8]) {
    buffer[0] = value.raw_bits();
}
