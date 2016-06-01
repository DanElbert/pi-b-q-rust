
use std::fs;
use libc;
use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::os::unix::fs::OpenOptionsExt;
use super::Packet;

pub enum ConnectionEvent {
    Data(Packet),
    Timeout
}

struct ThreadShare {
    signal: bool,
    tty_handle: fs::File
}

pub struct Connection<'a> {
    pub tty_path: &'a str,
    packet_reciever: Receiver<ConnectionEvent>,
    thread_share: Arc<Mutex<ThreadShare>>,
    thread_handle: Option<thread::JoinHandle<()>>
}

impl<'a> Connection<'a> {
    pub fn new(tty_path: &str) -> io::Result<Connection> {
        let (tx, rx) = channel::<ConnectionEvent>();

        let handle = try!(fs::OpenOptions::new()
                        .read(true)
                        .append(true)
                        //.custom_flags(libc::O_NOCTTY)
                        .open(tty_path));

        let signal = false;

        let thread_share = Arc::new(Mutex::new(ThreadShare {
            signal: signal,
            tty_handle: handle
        }));

        let inner_share = thread_share.clone();


        let thread = Some(thread::spawn(move || {
            loop {
                let mut share = inner_share.lock().expect("something bad");
                let mut buf = [0u8; 128];
                share.tty_handle.read_exact(&mut buf).expect("No Reading");

                if share.signal {
                    break;
                }
            }
            ()
        }));

        Ok(Connection {
            tty_path: tty_path,
            packet_reciever: rx,
            thread_handle: thread,
            thread_share: thread_share})
    }

    pub fn send(&mut self, p: &Packet) -> io::Result<usize> {
        let mut share = self.thread_share.lock().expect("Bad File");
        share.tty_handle.write(&p.data[..])
    }
}

impl<'a> Drop for Connection<'a> {
    fn drop(&mut self) {
        {
            let mut share = self.thread_share.lock().expect("Bad File");
            share.signal = true;
        }

        match self.thread_handle.take() {
            Some(th) => th.join().expect("thread was bad"),
            None => {}
        }
    }
}
