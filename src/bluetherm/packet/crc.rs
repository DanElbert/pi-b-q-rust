
pub fn compute_checksum(data: &[u8]) -> u16 {
    let mut crc = 0u16;
    for i in 0 .. data.len() {
        calculate_crc(data[i], &mut crc);
    }
    let mut ncrc = crc;
    ncrc = !ncrc;
    ncrc = ncrc & 0x0000FFFF;
    ncrc as u16
}

fn calculate_crc(p: u8, crc: &mut u16) {
    let mut temp_crc = *crc as u32;
    let mut word = p as u32;

    for _ in 0 .. 8 {
        let crcin: u32 = ((temp_crc ^ word) & 1) << 15;
        word = word >> 1;
        temp_crc = temp_crc >> 1;
        if crcin != 0 {
            temp_crc = temp_crc ^ (0xA001);
        }
    }

    *crc = temp_crc as u16;
}
