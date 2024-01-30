use simplelog::*;
//use std::fs::File;
use std::{ collections::LinkedList, fs::File};
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

#[macro_export]
macro_rules! say {
    ($fmt:literal, $($arg:expr),*) => {
            if cfg!(test){
                println!($fmt, $($arg),*);
            } else {
                crate::log::say_cb(&format!($fmt, $($arg),*), false);
            }
    };
    ($msg:expr) => {
            if cfg!(test){
                println!($msg);
            } else {
                crate::log::say_cb($msg, false);
            }
    };
}

#[macro_export]
macro_rules! verbose {
    ($fmt:literal, $($arg:expr),*) => {
            if cfg!(test){
                println!($fmt, $($arg),*);
            } else {
                crate::log::say_cb(&format!($fmt, $($arg),*), true);
            }

    };
    ($msg:expr) => {
            if cfg!(test){
                println!($msg);
            } else {
                crate::log::say_cb($msg, true);
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
// pub trait Sayer: Sync + Send {
//     fn say(& self, s: &str, verbose: bool);
// }
// struct NoSay;
// impl Sayer for NoSay {
//     fn say(& self, _s: &str, _verbose: bool) {}
// }

// static mut SAYER: &dyn Sayer = &NoSay;

// pub fn set_sayer(cb:  &'static impl Sayer ) {
//     unsafe { SAYER = cb };
// }
// pub fn say(s: &str) {
//     unsafe { SAYER.say(s, false) };
// }

// pub fn verbose(s: &str) {
//     unsafe{SAYER.say(s, true)};
// }
// use lazy_static::lazy_static;
// use std::sync::Mutex;

// type Callback = Box<dyn Fn(&str, bool) + Send + Sync>;

// lazy_static! {
//     static ref CALLBACK: Mutex<Option<Callback>> = Mutex::new(None);
// }

// pub fn register_callback(callback: Callback) {
//     let mut global_callback = CALLBACK.lock().unwrap();
//     *global_callback = Some(callback);
// }

// pub fn say_cb(s: &str, verbose: bool) {
//     let callback = CALLBACK.lock().unwrap();
//     if let Some(ref cb) = *callback {
//         cb(s, verbose);
//     }
// }

use once_cell::sync::OnceCell;
static SAY_CB: OnceCell<fn(&str,bool)> = OnceCell::new();
pub fn say_cb(s: &str, v:bool) {                                                                                               
       SAY_CB.get().unwrap()(s,v);
}
pub fn set_say_cb(cb: fn(&str,bool)) {                                                                                    
        SAY_CB.set(cb).unwrap();                                                                                        
    }