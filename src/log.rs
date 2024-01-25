use simplelog::*;
//use std::fs::File;
use std::{collections::LinkedList, fs::File, sync::Mutex};
#[macro_export]
macro_rules! trace {
    ($fmt:literal, $($arg:expr),*) => {
        #[cfg(debug_assertions)]
        {
            if cfg!(test){
                println!($fmt, $($arg),*);
            } else {
                log::trace!($fmt, $($arg),*);
            }
        }
    };
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        {
            if cfg!(test){
                println!($msg);
            } else {
                log::trace!($msg);
            }
        }
    };
}
pub(crate) use trace;
pub fn init_log() {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Error,
        ConfigBuilder::new()
            .add_filter_ignore_str("rustyline")
            .build(),
        File::create("my_rust_binary.log").unwrap(),
    )])
    .unwrap();
}
struct MyLog {
    buffer: LinkedList<String>,
    max_size: usize,
    current: String,
}
impl MyLog {
    fn new(max_size: usize) -> Self {
        Self {
            buffer: LinkedList::new(),
            max_size,
            current: String::new(),
        }
    }
}
impl std::io::Write for MyLog {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = std::str::from_utf8(buf).unwrap();
        if s == "\n" {
            self.buffer.push_back(self.current.clone());
            self.current.clear();
            if self.buffer.len() > self.max_size {
                self.buffer.pop_front();
            }
        } else {
            self.current.push_str(s);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        for s in &self.buffer {
            println!("{}", s);
        }
        Ok(())
    }
}

use once_cell::sync::{Lazy, OnceCell};

static SAY_CB: OnceCell<fn(&str)> = OnceCell::new();

pub fn set_say_cb(cb: fn(&str)) {
    SAY_CB.set(cb).unwrap();
}
pub fn say(s: &str) {
    SAY_CB.get().unwrap()(s);
}
