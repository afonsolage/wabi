use bevy::prelude::{debug, error, info, trace, warn, World};
use bevy_reflect::{
    erased_serde::__private::serde::de::DeserializeSeed,
    serde::{ReflectDeserializer, ReflectSerializer},
    FromReflect, Reflect, TypeRegistry,
};
use wabi_runtime_api::{
    mod_api::{log::LogMessage, query::Query, Action},
    WabiInstancePlatform,
};

use crate::reflect_query;

use super::WabiInstance;

pub(super) struct Context {
    instance: *mut WabiInstance,
    world: *mut World,
    registry: *const TypeRegistry,
}

impl Context {
    /// **This function should be called only on a callback from wasm module.**
    fn instance(&self) -> &'static mut WabiInstance {
        debug_assert!(!self.instance.is_null());

        // SAFETY: Context only runs after setup and in an exclusive system
        unsafe { &mut *self.instance }
    }

    /// **This function should be called only on a callback from wasm module.**
    fn world(&self) -> &'static mut World {
        debug_assert!(!self.world.is_null());

        // SAFETY: Context only runs after setup and in an exclusive system
        unsafe { &mut *self.world }
    }

    /// **This function should be called only on a callback from wasm module.**
    fn registry(&self) -> &'static TypeRegistry {
        debug_assert!(!self.registry.is_null());

        // SAFETY: Context only runs after setup and in an exclusive system
        unsafe { &*self.registry }
    }

    pub(super) fn setup(
        &mut self,
        world: &mut World,
        instance: &mut WabiInstance,
        registry: &TypeRegistry,
    ) {
        debug_assert!(self.instance.is_null());
        self.instance = instance;
        self.world = world;
        self.registry = registry;
    }

    pub(super) fn teardown(&mut self) {
        self.registry = std::ptr::null();
        self.instance = std::ptr::null_mut();
        self.world = std::ptr::null_mut();
    }

    fn deserialize_data(&self, len: u32) -> Box<dyn Reflect> {
        let buffer = self.instance().read_buffer(len);

        let reflect_deserializer = ReflectDeserializer::new(self.registry());

        let mut deserializer = {
            #[cfg(not(feature = "json"))]
            {
                rmp_serde::Deserializer::from_read_ref(buffer)
            }

            #[cfg(feature = "json")]
            {
                serde_json::Deserializer::from_slice(buffer)
            }
        };

        reflect_deserializer.deserialize(&mut deserializer).unwrap()
    }

    fn serialize_data(&self, data: Box<dyn Reflect>) -> Vec<u8> {
        #[cfg(not(feature = "json"))]
        {
            rmp_serde::encode::to_vec(&ReflectSerializer::new(&*data, self.registry())).unwrap()
        }
        #[cfg(feature = "json")]
        {
            serde_json::to_vec(&ReflectSerializer::new(&*data, self.registry())).unwrap()
        }
    }

    fn send_response(&self, data: Box<dyn Reflect>) -> u32 {
        let buffer = self.serialize_data(data);

        self.instance().write_buffer(&buffer);

        buffer.len() as u32
    }

    pub(super) fn process_action(&self, id: u32, len: u32, action: Action) -> u32 {
        assert_eq!(self.instance().id(), id);

        let data = self.deserialize_data(len);

        if action != Action::LOG {
            trace!("Received action: {:?}, data: {:?}", action, data);
        }

        let maybe_response = match action {
            Action::LOG => {
                let LogMessage { level, message } = LogMessage::from_reflect(&*data).unwrap();
                match level {
                    0 => trace!(message),
                    1 => debug!(message),
                    2 => info!(message),
                    3 => warn!(message),
                    4 => error!(message),
                    _ => error!("Invalid level received: {}. Message: ({})", level, message),
                };
                None
            }
            Action::QUERY => Some(self.process_query(Query::from_reflect(&*data).unwrap())),
            Action::TEST => {
                debug!("Received: {:?}", data);
                None
            }
            Action::INVALID => {
                error!("Invalid action received.");
                None
            }
        };

        if let Some(response) = maybe_response {
            trace!("Sending response: {:?}, data: {:?}", action, response);
            self.send_response(response)
        } else {
            0
        }
    }

    fn process_query(&self, query: Query) -> Box<dyn Reflect> {
        let result = reflect_query::dynamic_query(self.world(), query);
        result.clone_value()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            instance: std::ptr::null_mut(),
            world: std::ptr::null_mut(),
            registry: std::ptr::null(),
        }
    }
}
