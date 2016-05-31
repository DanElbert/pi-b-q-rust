use std::fmt;

bitflags! {
    pub flags MessageType: u8 {
        const NOTHING = 0u8,
        const RETRIEVE_INFO = 1u8,
        const SET_INFO = 2u8,
        const BUTTON_PRESS = 3u8,
        const RESERVED = 4u8,
        const SHUTDOWN = 5u8
    }
}

impl MessageType {
    pub fn raw_bits(&self) -> u8 {
        self.bits
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.bits)
    }
}
