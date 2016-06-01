
use std::io;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use serial::prelude::*;
use super::Packet;

pub enum ConnectionEvent {
    Data(Packet),
    Timeout
}

pub struct Connection<'a> {
    serial: &'a mut SerialPort
}

impl<'a> Connection<'a> {
    pub fn new(serial: &mut SerialPort) -> Connection {
        let (tx, rx) = channel::<ConnectionEvent>();
        let serial_wrapper = Arc::new(Mutex::new(serial));
        let thread_serial = serial_wrapper.clone();
        let thread = thread::spawn(|| {
            let serial = thread_serial.lock().unwrap();
            let mut buf = [0u8; 128];
            serial.read_exact(&mut buf[..]);
        });

        let c = Connection {serial: serial};
        c
    }

    pub fn send(&mut self, p: &Packet) -> io::Result<usize> {
        self.serial.write(&p.data[..])
    }
}
