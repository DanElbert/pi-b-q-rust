mod thread_guard;

use std::fs;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};
use std::ops::Deref;

use nonblock::NonBlockingReader;
use self::thread_guard::*;
use super::Packet;

pub enum ConnectionEvent {
    Packet(Packet),
    InvalidPacket(Packet),
    ReadError(io::Error),
    BadData(Vec<u8>)
}

impl fmt::Display for ConnectionEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ConnectionEvent::Packet(ref p) => write!(f, "{}", p),
            &ConnectionEvent::InvalidPacket(ref p) => write!(f, "Invalid! {}", p),
            &ConnectionEvent::ReadError(ref err) => write!(f, "ERROR! [{}]", err),
            &ConnectionEvent::BadData(ref data) => write!(f, "BAD DATA! [{:?}]", data)
        }
    }
}

pub struct Connection<'a> {
    pub tty_path: &'a str,
    tty_writer: fs::File,
    kill_thread_signal: Arc<Mutex<bool>>,
    processor_thread_handle: Option<ThreadHandle<()>>,
    reader_thread_handle: Option<ThreadHandle<()>>,
}

impl<'a> Connection<'a> {
    pub fn new(tty_path: &str) -> io::Result<Connection> {
        let (tx, rx) = channel::<ConnectionEvent>();

        let kill_signal = Arc::new(Mutex::new(false));

        println!("here? 1");

        let tty_reader = try!(fs::OpenOptions::new()
                        .create(false)
                        .read(true)
                        .write(false)
                        .open(tty_path));

        println!("here? 2");

        let tty_writer = try!(fs::OpenOptions::new()
                        .create(false)
                        .read(false)
                        .write(true)
                        .open(tty_path));

        println!("here? 3");

        let nonblocking_reader = NonBlockingReader::from_fd(tty_reader).expect("error creating non-blocking reader");

        let reader_thread = build_connection_read_thread(nonblocking_reader, tx, kill_signal.clone());
        let processor_thread = build_processor_thread(rx, kill_signal.clone());

        Ok(Connection {
            tty_path: tty_path,
            tty_writer: tty_writer,
            kill_thread_signal: kill_signal,
            processor_thread_handle: Some(processor_thread),
            reader_thread_handle: Some(reader_thread)
        })
    }

    pub fn send(&mut self, p: &Packet) -> io::Result<usize> {
        self.tty_writer.write(&p.data[..])
    }

    pub fn is_ok(&self) -> bool {
        let mut ret = true;

        match &self.processor_thread_handle {
            &Some(ref h) => {
                ret = ret && match h.status.lock().unwrap().deref() {
                    &ThreadStatus::Ok => true,
                    _ => false
                }
            },
            _ => {}
        }

        match &self.reader_thread_handle {
            &Some(ref h) => {
                ret = ret && match h.status.lock().unwrap().deref() {
                    &ThreadStatus::Ok => true,
                    _ => false
                }
            },
            _ => {}
        }
        ret
    }
}

impl<'a> Drop for Connection<'a> {
    fn drop(&mut self) {

        {
            *self.kill_thread_signal.lock().unwrap() = true;
        }

        match self.reader_thread_handle.take() {
            Some(th) => {
                th.handle.join().unwrap();
            },
            None => {}
        }

        match self.processor_thread_handle.take() {
            Some(th) => {
                th.handle.join().unwrap();
            },
            None => {}
        }
    }
}

fn build_processor_thread(receiver: Receiver<ConnectionEvent>, kill_signal: Arc<Mutex<bool>>) -> ThreadHandle<()> {
    guard_thread(move || {
        while !*kill_signal.lock().unwrap() {
            match receiver.recv() {
                Err(_) => {
                    break;
                },
                Ok(evt) => {
                    println!("Got: {}", evt);
                }
            }
        }
        ()
    })
}

fn build_connection_read_thread(mut reader: NonBlockingReader<fs::File>, sender: Sender<ConnectionEvent>, kill_signal: Arc<Mutex<bool>>) -> ThreadHandle<()> {
    guard_thread(move || {
        let mut packet_buffer: Vec<u8> = Vec::new();
        let mut read_buffer: Vec<u8> = Vec::new();
        let mut last_read = Instant::now();

        while !*kill_signal.lock().unwrap() {

            if packet_buffer.len() > 0 && last_read.elapsed().as_secs() > 2 {
                let evt = ConnectionEvent::BadData(packet_buffer.clone());
                sender.send(evt).unwrap();
                packet_buffer.clear();
            }

            read_buffer.clear();

            match reader.read_available(&mut read_buffer) {
                Err(e) => {
                    let evt = ConnectionEvent::ReadError(e);
                    sender.send(evt).unwrap();
                },
                Ok(bytes) if bytes > 0 => {
                    last_read = Instant::now();
                    for x in 0 .. bytes {
                        packet_buffer.push(read_buffer[x]);
                    }
                },
                Ok(_) => {} // do nothing for 0 bytes read
            }

            while packet_buffer.len() >= 128 {
                let data: Vec<u8> = packet_buffer.drain(0..128).collect();
                let p = Packet::from_bytes(&data);

                if p.is_checksum_valid() {
                    let evt = ConnectionEvent::Packet(p);
                    sender.send(evt).unwrap();
                } else {
                    let evt = ConnectionEvent::InvalidPacket(p);
                    sender.send(evt).unwrap();
                }
            }

            thread::sleep(Duration::from_millis(200));
        }
        ()
    })
}
