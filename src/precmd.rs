use std::time::Duration;
use std::thread;

pub fn display()-> String {
    thread::sleep(Duration::from_secs(0));
    "%F{074}%~%f".to_string()
}
