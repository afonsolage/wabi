use std::io::Write;

use bevy_reflect::{serde::ReflectSerializer, FromReflect, Reflect, TypeRegistry};

use wabi_mod_api::{
    ecs::Component,
    log::LogMessage,
    query::{QueryFetch, QueryFetchItem},
    registry::create_type_registry,
    Action,
};

static mut INSTANCE_DATA: InstanceData = InstanceData {
    id: 0,
    buffer: vec![],
    registry: None,
};

fn get_instance_data() -> &'static mut InstanceData {
    // SAFETY: Wasm modules are single threaded
    unsafe { &mut INSTANCE_DATA }
}

#[derive(Default)]
struct InstanceData {
    id: u32,
    buffer: Vec<u8>,
    registry: Option<TypeRegistry>,
}

impl InstanceData {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry.as_ref().unwrap()
    }
}

struct ActionBufferWriter {
    len: usize,
    action: u8,
}

impl ActionBufferWriter {
    fn new(action: Action) -> Self {
        Self {
            len: 0,
            action: action as u8,
        }
    }
}

impl std::io::Write for ActionBufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let begin = self.len;
        self.len = begin + buf.len();

        unsafe {
            INSTANCE_DATA.buffer[begin..self.len].copy_from_slice(buf);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unsafe {
            __wabi_process_action(get_instance_data().id, self.len, self.action);
        }
        Ok(())
    }
}

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

#[link(wasm_import_module = "wabi")]
extern "C" {
    fn __wabi_process_action(id: u32, len: usize, action: u8);
}

#[no_mangle]
pub extern "C" fn __wabi_entry_point() {
    // crate::test::run();

    let log = LogMessage {
        level: 3,
        message: "Something!".into(),
    };

    let dynstryct = log.as_reflect();

    let test = QueryFetch {
        items: vec![QueryFetchItem::ReadOnly(
            Component::from_reflect(dynstryct).unwrap(),
        )],
    };

    send_action(&test, Action::TEST);
}

const PAGE_SIZE: usize = 65536;

#[no_mangle]
pub extern "C" fn __wabi_alloc(id: u32) -> i32 {
    // SAFETY: this function will be called only by host, so only one mutable access at any given time.
    unsafe {
        INSTANCE_DATA = InstanceData {
            id,
            buffer: vec![0u8; PAGE_SIZE as usize],
            registry: Some(create_type_registry()),
        };

        INSTANCE_DATA.buffer.as_ptr() as i32
    }
}

pub fn send_action<T: Reflect>(data: &T, action: Action) {
    let mut writer = ActionBufferWriter::new(action);

    let data = rmp_serde::encode::write(
        &mut writer,
        &ReflectSerializer::new(data, get_instance_data().get_registry()),
    );

    match data {
        Ok(()) => writer.flush().expect("Should never fail"),
        Err(err) if action != Action::LOG => error(&format!("Failed to send message: {:?}", err)),
        _ => panic!("Failed to serialize a log message, so, no log message."),
    }
}
