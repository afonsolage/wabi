use std::io::Write;

use bevy_reflect::{
    erased_serde::__private::serde::de::DeserializeSeed,
    serde::{ReflectDeserializer, ReflectSerializer},
    Reflect, TypeRegistry,
};
use wabi_mod_api::{registry::create_type_registry, Action};

use crate::wabi::error;

const PAGE_SIZE: usize = 65536;

static mut INSTANCE_DATA: InstanceData = InstanceData {
    id: 0,
    buffer: vec![],
    registry: None,
};

pub(crate) fn get_instance_data() -> &'static mut InstanceData {
    // SAFETY: Wasm modules are single threaded
    unsafe { &mut INSTANCE_DATA }
}

#[derive(Default)]
pub(crate) struct InstanceData {
    pub id: u32,
    pub buffer: Vec<u8>,
    pub registry: Option<TypeRegistry>,
}

impl InstanceData {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry.as_ref().unwrap()
    }
}

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

pub fn send_action<T: Reflect>(data: &T, action: Action) -> Option<Box<dyn Reflect>> {
    let mut writer = HostWriter::new(action);

    let registry = get_instance_data().get_registry();

    let reflect_serializer = ReflectSerializer::new(data, registry);
    let data = {
        #[cfg(not(feature = "json"))]
        {
            rmp_serde::encode::write(&mut writer, &reflect_serializer)
        }
        #[cfg(feature = "json")]
        {
            serde_json::to_writer(&mut writer, &reflect_serializer)
        }
    };

    match data {
        Ok(()) => writer.flush().expect("Should never fail"),
        Err(err) if action != Action::LOG => error(&format!("Failed to send message: {:?}", err)),
        _ => panic!("Failed to serialize a log message, so, no log message."),
    }

    if writer.len == 0 {
        None
    } else {
        let reflect_deserializer = ReflectDeserializer::new(registry);
        let mut deserializer = {
            #[cfg(not(feature = "json"))]
            {
                rmp_serde::Deserializer::from_read_ref(writer.response_buffer())
            }
            #[cfg(feature = "json")]
            {
                serde_json::Deserializer::from_slice(writer.response_buffer())
            }
        };

        match reflect_deserializer.deserialize(&mut deserializer) {
            Ok(response) => Some(response),
            Err(err) => {
                error(format!("Failed to receive response: {:?}", err));
                None
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct HostWriter {
    len: usize,
    action: u8,
}

impl HostWriter {
    pub fn new(action: Action) -> Self {
        Self {
            len: 0,
            action: action as u8,
        }
    }
}

impl HostWriter {
    fn response_buffer(&self) -> &[u8] {
        assert!(self.len > 0);

        unsafe { &INSTANCE_DATA.buffer[..self.len] }
    }
}

impl std::io::Write for HostWriter {
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
            self.len =
                __wabi_process_action(get_instance_data().id, self.len, self.action) as usize;
        }
        Ok(())
    }
}

#[link(wasm_import_module = "wabi")]
extern "C" {
    fn __wabi_process_action(id: u32, len: usize, action: u8) -> u32;
}
