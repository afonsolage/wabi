use std::io::Write;

use bevy_reflect::{
    erased_serde::__private::serde::de::DeserializeSeed,
    serde::{ReflectSerializer, UntypedReflectDeserializer},
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

trait HostWriter: Write + Sized {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn response_buffer(&self) -> &[u8] {
        assert!(self.is_empty());
        unsafe { &INSTANCE_DATA.buffer[..self.len()] }
    }

    fn send(mut self, data: &dyn Reflect) -> Option<Box<dyn Reflect>> {
        let registry = get_instance_data().get_registry();

        let reflect_serializer = ReflectSerializer::new(data, registry);
        let data = {
            #[cfg(not(feature = "json"))]
            {
                rmp_serde::encode::write(&mut self, &reflect_serializer)
            }
            #[cfg(feature = "json")]
            {
                serde_json::to_writer(&mut self, &reflect_serializer)
            }
        };

        match data {
            Ok(()) => self.flush().expect("Should never fail"),
            Err(err) => error(&format!("Failed to send message: {:?}", err)),
        }

        if self.is_empty() {
            None
        } else {
            let reflect_deserializer = UntypedReflectDeserializer::new(registry);
            let mut deserializer = {
                #[cfg(not(feature = "json"))]
                {
                    rmp_serde::Deserializer::from_read_ref(self.response_buffer())
                }
                #[cfg(feature = "json")]
                {
                    serde_json::Deserializer::from_slice(self.response_buffer())
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
}

pub fn send_action(data: &dyn Reflect, action: Action) -> Option<Box<dyn Reflect>> {
    ActionWriter::new(action).send(data)
}

#[derive(Default)]
struct ActionWriter {
    len: usize,
    action: u8,
}

impl ActionWriter {
    pub fn new(action: Action) -> Self {
        Self {
            len: 0,
            action: action as u8,
        }
    }
}

impl HostWriter for ActionWriter {
    fn len(&self) -> usize {
        self.len
    }
}

impl std::io::Write for ActionWriter {
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
