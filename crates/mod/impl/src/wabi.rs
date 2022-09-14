use wabi_mod_api::{log::LogMessage, Action};

use crate::{io::send_action, query};

pub fn trace(message: impl ToString) {
    log::<0>(message.to_string());
}

pub fn debug(message: impl ToString) {
    log::<1>(message.to_string());
}

pub fn info(message: impl ToString) {
    log::<2>(message.to_string());
}

pub fn warn(message: impl ToString) {
    log::<3>(message.to_string());
}

pub fn error(message: impl ToString) {
    log::<4>(message.to_string());
}

pub fn log<const L: u8>(message: String) {
    send_action(&LogMessage { level: L, message }, Action::LOG);
}

#[no_mangle]
pub extern "C" fn __wabi_entry_point() {
    let result = query::query(&["bevy_transform::components::transform::Transform"]);
    debug(format!("Query result: {:?}", result));
}

macro_rules! unwrap {
    ($tt:expr) => {
        match $tt {
            Some(t) => t,
            None => {
                crate::wabi::error(format!("Failed unwrap at: {}:{}", file!(), line!()));
                unreachable!()
            }
        }
    };
}

pub(crate) use unwrap;
