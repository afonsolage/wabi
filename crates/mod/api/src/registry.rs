use std::ops::Range;

use bevy::utils::HashSet;
use bevy_reflect::TypeRegistry;

use crate::{
    log::LogMessage,
    query::{Query, QueryFetch},
};

pub fn create_type_registry() -> TypeRegistry {
    let mut registry = TypeRegistry::new();

    registry.register::<Range<f32>>();
    registry.register::<HashSet<String>>();
    registry.register::<String>();
    registry.register::<Option<String>>();

    registry.register::<LogMessage>();
    registry.register::<Query>();
    registry.register::<QueryFetch>();

    registry
}
