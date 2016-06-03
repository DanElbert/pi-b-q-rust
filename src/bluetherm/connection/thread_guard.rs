use std::sync::{Arc, Mutex};
use std::thread;

pub enum ThreadStatus {
    Ok,
    Done,
    Err
}

pub struct ThreadHandle<T> {
    pub status: Arc<Mutex<ThreadStatus>>,
    pub handle: thread::JoinHandle<T>
}


pub fn guard_thread<F, T>(f: F) -> ThreadHandle<T>
    where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {

    let status = Arc::new(Mutex::new(ThreadStatus::Ok));
    let tstatus = status.clone();

    let inner_handle = thread::spawn(f);
    let handle = thread::spawn(move || {
        let ret = inner_handle.join();
        match ret {
            Ok(x) => {
                *tstatus.lock().unwrap() = ThreadStatus::Done;
                x
            },
            Err(e) => {
                *tstatus.lock().unwrap() = ThreadStatus::Err;
                panic!(e);
            }
        }
    });

    ThreadHandle{ handle: handle, status: status }
}
