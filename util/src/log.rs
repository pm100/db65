use simplelog::*;

use std::{collections::LinkedList, fs::File};
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
#[allow(dead_code)]
struct MyLog {
    buffer: LinkedList<String>,
    max_size: usize,
    current: String,
}
#[allow(dead_code)]
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
