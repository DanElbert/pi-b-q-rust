mod thread_guard;

use std::fs;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
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
    BadData(Vec<u8>),
    Heartbeat
}

impl fmt::Display for ConnectionEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ConnectionEvent::Packet(ref p) => write!(f, "{}", p),
            &ConnectionEvent::InvalidPacket(ref p) => write!(f, "Invalid! {}", p),
            &ConnectionEvent::ReadError(ref err) => write!(f, "ERROR! [{}]", err),
            &ConnectionEvent::BadData(ref data) => write!(f, "BAD DATA! [{:?}]", data),
            &ConnectionEvent::Heartbeat => write!(f, "Tick Tock")
        }
    }
}

pub struct Connection {
    pub tty_path: String,
    tty_writer: fs::File,
    receiver: Receiver<ConnectionEvent>,
    kill_thread_signal: Arc<Mutex<bool>>,
    reader_thread_handle: Option<ThreadHandle<()>>,
}

impl Connection {
    pub fn new(tty_path: &str, heartbeat_milliseconds: Option<u64>) -> io::Result<Connection> {
        let (tx, rx) = channel::<ConnectionEvent>();

        let kill_signal = Arc::new(Mutex::new(false));

        let tty_reader = try!(fs::OpenOptions::new()
                        .create(false)
                        .read(true)
                        .write(false)
                        .open(tty_path));

        let tty_writer = try!(fs::OpenOptions::new()
                        .create(false)
                        .read(false)
                        .write(true)
                        .open(tty_path));

        let nonblocking_reader = NonBlockingReader::from_fd(tty_reader).expect("error creating non-blocking reader");

        let reader_thread = build_connection_read_thread(nonblocking_reader, tx, heartbeat_milliseconds, kill_signal.clone());

        Ok(Connection {
            tty_path: tty_path.to_string(),
            tty_writer: tty_writer,
            receiver: rx,
            kill_thread_signal: kill_signal,
            reader_thread_handle: Some(reader_thread)
        })
    }

    pub fn send(&mut self, p: &Packet) -> io::Result<usize> {
        match self.is_ok() {
            true => self.tty_writer.write(&p.data[..]),
            false => Err(io::Error::new(io::ErrorKind::ConnectionAborted, "worker threads have stopped"))
        }
    }

    pub fn wait(&self) -> io::Result<ConnectionEvent> {
        match self.is_ok() {
            false => Err(io::Error::new(io::ErrorKind::ConnectionAborted, "worker threads have stopped")),
            true => {
                Ok(self.receiver.recv().unwrap())
            }
        }
    }

    pub fn get_events(&self) -> io::Result<Vec<ConnectionEvent>> {
        match self.is_ok() {
            false => Err(io::Error::new(io::ErrorKind::ConnectionAborted, "worker threads have stopped")),
            true => {
                let mut data: Vec<ConnectionEvent> = vec!();

                loop {
                    match self.receiver.try_recv() {
                        Ok(evt) => { data.push(evt) },
                        Err(TryRecvError::Empty) => { break; },
                        Err(TryRecvError::Disconnected) => { panic!("channels should not disconnect") }
                    }
                }

                Ok(data)
            }
        }
    }

    pub fn is_ok(&self) -> bool {
        let mut ret = true;

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

impl Drop for Connection {
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
    }
}


fn build_connection_read_thread(mut reader: NonBlockingReader<fs::File>, sender: Sender<ConnectionEvent>, heartbeat: Option<u64>, kill_signal: Arc<Mutex<bool>>) -> ThreadHandle<()> {
    guard_thread("reader_thread", move || {
        let mut packet_buffer: Vec<u8> = Vec::new();
        let mut read_buffer: Vec<u8> = Vec::new();
        let mut last_read = Instant::now();
        let mut last_heartbeat = Instant::now();

        while !*kill_signal.lock().unwrap() {

            println!("a");

            if packet_buffer.len() > 0 && last_read.elapsed().as_secs() > 2 {
                let evt = ConnectionEvent::BadData(packet_buffer.clone());
                sender.send(evt).unwrap();
                packet_buffer.clear();
            }

            println!("b");

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

            println!("c");

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

            println!("d");

            match heartbeat {
                Some(ms) => {
                    if last_heartbeat.elapsed() >= Duration::from_millis(ms) {
                        let evt = ConnectionEvent::Heartbeat;
                        sender.send(evt).unwrap();
                        last_heartbeat = Instant::now();
                    }
                },
                None => {}
            }

            println!("e");

            thread::sleep(Duration::from_millis(100));

            println!("f");
        }
        ()
    })
}
