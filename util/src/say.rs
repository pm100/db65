#[macro_export]
macro_rules! say {
    ($fmt:literal, $($arg:expr),*) => {
            if cfg!(test){
                println!($fmt, $($arg),*);
            } else {
                util::say::say_cb(&format!($fmt, $($arg),*), false);
            }
    };
    ($msg:expr) => {
            if cfg!(test){
                println!($msg);
            } else {
                util::say::say_cb($msg, false);
            }
    };
}

#[macro_export]
macro_rules! verbose {
    ($fmt:literal, $($arg:expr),*) => {
            if cfg!(test){
                println!($fmt, $($arg),*);
            } else {
                util::say::say_cb(&format!($fmt, $($arg),*), true);
            }

    };
    ($msg:expr) => {
            if cfg!(test){
                println!($msg);
            } else {
                util::say::say_cb($msg, true);
            }
    };
}

use once_cell::sync::OnceCell;
pub static SAY_CB: OnceCell<fn(&str, bool)> = OnceCell::new();
pub fn say_cb(s: &str, v: bool) {
    SAY_CB.get().unwrap()(s, v);
}
pub fn set_say_cb(cb: fn(&str, bool)) {
    SAY_CB.set(cb).unwrap();
}
