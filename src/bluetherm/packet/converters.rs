use std::str;

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
    match str::from_utf8(data) {
        Ok(value) => value.to_string(),
        Err(_) => "".to_string()
    }
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
        value = (value / 300_000.0f32) - 300.0f32;
    }

    value
}
