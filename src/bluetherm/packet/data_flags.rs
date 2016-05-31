use std::fmt;

bitflags! {
    pub flags DataFlags: u16 {
        const NONE = 0u16,
        const SERIAL_NUMBER = 1u16,
        const PROBE_NAMES = 2u16,
        const SENSOR_1_TEMPERATURE = 4u16,
        const SENSOR_1_HIGH_LIMIT = 8u16,
        const SENSOR_1_LOW_LIMIT = 16u16,
        const SENSOR_1_TRIM = 32u16,
        const SENSOR_2_TEMPERATURE = 64u16,
        const SENSOR_2_HIGH_LIMIT = 128u16,
        const SENSOR_2_LOW_LIMIT = 256u16,
        const SENSOR_2_TRIM = 512u16,
        const BATTERY_CONDITION = 1024u16,
        const CAL_VALUE_1 = 2048u16,
        const CAL_VALUE_2 = 4096u16,
        const CAL_VALUE_3 = 8192u16,
        const FIRMWARE_VERSION = 16384u16,
        const TYPES = 32768u16,
        const DEFAULT = SERIAL_NUMBER.bits | PROBE_NAMES.bits | SENSOR_1_TEMPERATURE.bits | SENSOR_2_TEMPERATURE.bits | BATTERY_CONDITION.bits
    }
}

impl DataFlags {
    pub fn raw_bits(&self) -> u16 {
        self.bits
    }
}

impl fmt::Display for DataFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.bits)
    }
}
