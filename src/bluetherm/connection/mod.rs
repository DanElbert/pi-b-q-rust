mod thread_guard;

use std::fmt;
use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use std::ops::Deref;

use serial::prelude::*;
use serial;
use self::thread_guard::*;
use super::Packet;

pub enum ConnectionEvent {
    Packet(Packet),
    InvalidPacket(Packet),
    ReadError(io::Error),
    WriteError(io::Error),
    Heartbeat
}

impl fmt::Display for ConnectionEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ConnectionEvent::Packet(ref p) => write!(f, "{}", p),
            &ConnectionEvent::InvalidPacket(ref p) => write!(f, "Invalid! {}", p),
            &ConnectionEvent::ReadError(ref err) => write!(f, "READ ERROR! [{:?}]", err),
            &ConnectionEvent::WriteError(ref err) => write!(f, "WRITE ERROR! [{:?}]", err),
            &ConnectionEvent::Heartbeat => write!(f, "Tick Tock")
        }
    }
}

pub struct Connection {
    pub tty_path: String,
    event_receiver: Receiver<ConnectionEvent>,
    packet_sender: Sender<Packet>,
    kill_thread_signal: Arc<Mutex<bool>>,
    reader_thread_handle: Option<ThreadHandle<()>>,
}

impl Connection {
    pub fn new(tty_path: &str, heartbeat_milliseconds: Option<u64>) -> io::Result<Connection> {
        let (event_sender, event_receiver) = channel::<ConnectionEvent>();
        let (packet_sender, packet_receiver) = channel::<Packet>();

        let kill_signal = Arc::new(Mutex::new(false));

        let reader_thread = build_connection_read_thread(tty_path.to_string(), event_sender, packet_receiver, heartbeat_milliseconds, kill_signal.clone());

        Ok(Connection {
            tty_path: tty_path.to_string(),
            event_receiver: event_receiver,
            packet_sender: packet_sender,
            kill_thread_signal: kill_signal,
            reader_thread_handle: Some(reader_thread)
        })
    }

    pub fn send(&mut self, p: Packet) -> io::Result<()> {
        match self.is_ok() {
            true => {
                self.packet_sender.send(p).unwrap();
                Ok(())
            },
            false => Err(io::Error::new(io::ErrorKind::ConnectionAborted, "worker threads have stopped"))
        }
    }

    pub fn wait(&self) -> io::Result<ConnectionEvent> {
        match self.is_ok() {
            false => Err(io::Error::new(io::ErrorKind::ConnectionAborted, "worker threads have stopped")),
            true => {
                Ok(self.event_receiver.recv().unwrap())
            }
        }
    }

    pub fn get_events(&self) -> io::Result<Vec<ConnectionEvent>> {
        match self.is_ok() {
            false => Err(io::Error::new(io::ErrorKind::ConnectionAborted, "worker threads have stopped")),
            true => {
                let mut data: Vec<ConnectionEvent> = vec!();

                loop {
                    match self.event_receiver.try_recv() {
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


fn build_connection_read_thread(tty_path: String, event_sender: Sender<ConnectionEvent>, packet_receiver: Receiver<Packet>, heartbeat: Option<u64>, kill_signal: Arc<Mutex<bool>>) -> ThreadHandle<()> {
    guard_thread("reader_thread", move || {
        let mut serial = match serial::open(&tty_path) {
            Err(e) => {panic!(format!("Unable to create serial port: {}", e)) },
            Ok(s) => s
        };

        match serial.set_timeout(Duration::from_millis(2000)) {
            Err(e) => { panic!(format!("Unable to set serial timeout: {}", e)) },
            Ok(_) => {}
        };

        let mut packet_buffer: Vec<u8> = Vec::new();
        let mut read_buffer: Vec<u8> = vec![0u8; 128];
        let mut last_read = Instant::now();
        let mut last_heartbeat = Instant::now();

        while !*kill_signal.lock().unwrap() {

            if packet_buffer.len() > 0 && last_read.elapsed().as_secs() > 4 {
                let evt = ConnectionEvent::ReadError(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid packet data: {:?}", packet_buffer.clone())));
                event_sender.send(evt).unwrap();
                packet_buffer.clear();
            };

            match packet_receiver.try_recv() {
                Ok(p) => {
                    println!("going to write...");
                    match serial.write_all(&p.data) {
                        Ok(_) => {},
                        Err(e) => {
                            let evt = ConnectionEvent::WriteError(e);
                            event_sender.send(evt).unwrap();
                        }
                    }
                },
                Err(TryRecvError::Empty) => {  },
                Err(TryRecvError::Disconnected) => { panic!("channels should not disconnect") }
            }

            match serial.read(&mut read_buffer) {
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {},
                Err(e) => {
                    let evt = ConnectionEvent::ReadError(e);
                    event_sender.send(evt).unwrap();
                },
                Ok(bytes) if bytes > 0 => {
                    println!("bytes: {}", bytes);
                    last_read = Instant::now();
                    for x in 0 .. bytes {
                        packet_buffer.push(read_buffer[x]);
                    }
                },
                Ok(bytes) => { println!("bytes: {}", bytes); } // do nothing for 0 bytes read
            };

            while packet_buffer.len() >= 128 {
                let data: Vec<u8> = packet_buffer.drain(0..128).collect();
                let p = Packet::from_bytes(&data);

                if p.is_checksum_valid() {
                    let evt = ConnectionEvent::Packet(p);
                    event_sender.send(evt).unwrap();
                } else {
                    let evt = ConnectionEvent::InvalidPacket(p);
                    event_sender.send(evt).unwrap();
                }
            }

            match heartbeat {
                Some(ms) => {
                    if last_heartbeat.elapsed() >= Duration::from_millis(ms) {
                        let evt = ConnectionEvent::Heartbeat;
                        event_sender.send(evt).unwrap();
                        last_heartbeat = Instant::now();
                    }
                },
                None => {}
            }

            thread::sleep(Duration::from_millis(250));
        }
        ()
    })
}
