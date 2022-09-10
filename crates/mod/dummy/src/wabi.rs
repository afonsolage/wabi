use std::io::Write;

use bevy_reflect::{serde::ReflectSerializer, Reflect, TypeRegistry};

use wabi_api::{create_type_registry, Action};

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

// TODO: I need this in order to __wabi_send_buffer don't get removed when compiling.
// Probably I'm missing something here, but I let this task for my future me.
// #[no_mangle]
// pub unsafe extern "C" fn __wabi_dont_call_me() {
//     __wabi_process_action(INSTANCE_ID, 0);
// }

pub fn debug(message: &String) {
    send_action(message, Action::DEBUG);
}

#[link(wasm_import_module = "wabi")]
extern "C" {
    fn __wabi_process_action(id: u32, len: usize, action: u8);
}

#[no_mangle]
pub extern "C" fn __wabi_main() {
    // crate::test::run();
    debug(&"Running main!".to_string());
    debug(&"Will this work?".to_string());
    debug(&"Even if I send many messages".to_string());
}

const PAGE_SIZE: usize = 65536;

#[no_mangle]
pub extern "C" fn __wabi_init(id: u32) -> i32 {
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
    rmp_serde::encode::write(
        &mut writer,
        &ReflectSerializer::new(data, get_instance_data().get_registry()),
    )
    .expect("Should never fail");
    writer.flush().expect("Should never fail");
}
