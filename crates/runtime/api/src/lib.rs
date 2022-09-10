use std::ops::Range;

use bevy::{prelude::info, utils::HashSet};

// We need both, since bevy::prelude rename TypeRegistry
use bevy_reflect::{
    erased_serde::__private::serde::de::DeserializeSeed, serde::ReflectDeserializer, TypeRegistry,
};

pub trait WabiRuntime {
    fn new() -> Self;
    fn load_mod(&mut self, name: String, buffer: &[u8]);
    fn run(&mut self, name: &str) -> i32;
    fn add_export<F>(&mut self, name: String, f: F)
    where
        F: Fn();
}

#[derive(num_enum::FromPrimitive)]
#[repr(u8)]
pub enum Action {
    DEBUG,

    #[default]
    INVALID = 255,
}

pub fn process_action(buffer: &[u8], action: Action) {
    let type_registry = create_type_registry();
    let reflect_deserializer = ReflectDeserializer::new(&type_registry);
    let mut deserializer = rmp_serde::Deserializer::from_read_ref(buffer);
    let value = reflect_deserializer.deserialize(&mut deserializer).unwrap();

    match action {
        Action::DEBUG => {
            let message = value.downcast_ref::<String>().unwrap();
            info!("{}", message);
        }
        Action::INVALID => info!("Invalid action received."),
    }
}

pub fn create_type_registry() -> TypeRegistry {
    let mut registry = TypeRegistry::new();

    registry.register::<Range<f32>>();
    registry.register::<HashSet<String>>();
    registry.register::<String>();
    registry.register::<Option<String>>();

    registry
}
