use std::fmt::Debug;

use bevy_reflect::{FromReflect, Reflect};

use crate::reflect_proxy;

#[derive(Reflect, FromReflect, Default, Debug, Clone, Copy)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

reflect_proxy::impl_type!(Component);
