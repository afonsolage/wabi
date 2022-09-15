use std::ops::Range;

use bevy::utils::HashSet;
use bevy_reflect::TypeRegistry;

use crate::{
    log::LogMessage,
    query::{Query, QueryFetch, QueryFetchItem},
};

pub fn create_type_registry() -> TypeRegistry {
    let mut registry = TypeRegistry::new();

    register_bevy_types(&mut registry);
    register_api_types(&mut registry);

    registry
}

pub fn register_api_types(registry: &mut TypeRegistry) {
    registry.register::<LogMessage>();
    registry.register::<Query>();
    registry.register::<QueryFetch>();
    registry.register::<QueryFetchItem>();
}

pub fn register_bevy_types(registry: &mut TypeRegistry) {
    registry.register::<Range<f32>>();
    registry.register::<HashSet<String>>();
    registry.register::<String>();
    registry.register::<Option<String>>();

    registry.register::<bevy_math::IVec2>();
    registry.register::<bevy_math::IVec3>();
    registry.register::<bevy_math::IVec4>();
    registry.register::<bevy_math::UVec2>();
    registry.register::<bevy_math::UVec3>();
    registry.register::<bevy_math::UVec4>();
    registry.register::<bevy_math::DVec2>();
    registry.register::<bevy_math::DVec3>();
    registry.register::<bevy_math::DVec4>();
    registry.register::<bevy_math::BVec2>();
    registry.register::<bevy_math::BVec3>();
    registry.register::<bevy_math::BVec3A>();
    registry.register::<bevy_math::BVec4>();
    registry.register::<bevy_math::BVec4A>();
    registry.register::<bevy_math::Vec2>();
    registry.register::<bevy_math::Vec3>();
    registry.register::<bevy_math::Vec3A>();
    registry.register::<bevy_math::Vec4>();
    registry.register::<bevy_math::DAffine2>();
    registry.register::<bevy_math::DAffine3>();
    registry.register::<bevy_math::Affine2>();
    registry.register::<bevy_math::Affine3A>();
    registry.register::<bevy_math::DMat2>();
    registry.register::<bevy_math::DMat3>();
    registry.register::<bevy_math::DMat4>();
    registry.register::<bevy_math::Mat2>();
    registry.register::<bevy_math::Mat3>();
    registry.register::<bevy_math::Mat3A>();
    registry.register::<bevy_math::Mat4>();
    registry.register::<bevy_math::DQuat>();
    registry.register::<bevy_math::Quat>();
}

// fn fix_type_name_platform_dependent(registry: &mut TypeRegistry) {
//     let fix_mapping = vec![("glam::f32::scalar", "glam::f32::sse2")];

//     let mut to_add = vec![];

//     for item in registry.iter() {
//         for &(a, b) in fix_mapping.iter() {
//             if item.type_name().starts_with(a) {
//                 to_add.push((b, item.type_id()));
//             } else if item.type_name().starts_with(b) {
//                 to_add.push((a, item.type_id()));
//             }
//         }
//     }

//     registry.add_registration(registration)
// }
