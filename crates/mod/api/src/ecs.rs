use std::fmt::Debug;

use bevy_reflect::{DynamicEnum, DynamicStruct, Enum, FromReflect, Reflect, ReflectRef, Struct};

#[derive(Reflect, FromReflect, Default, Debug, Clone, Copy)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

#[derive(Reflect, Default, Debug)]
pub struct DynStruct(DynamicStruct);

impl FromReflect for DynStruct {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        if let ReflectRef::Struct(value) = reflect.reflect_ref() {
            Some(Self(value.clone_dynamic()))
        } else {
            None
        }
    }
}

#[derive(Reflect, Default)]
pub struct DynEnum(DynamicEnum);

impl FromReflect for DynEnum {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        if let ReflectRef::Enum(value) = reflect.reflect_ref() {
            Some(Self(value.clone_dynamic()))
        } else {
            None
        }
    }
}

#[derive(Reflect, FromReflect, Default)]
pub enum Component {
    #[default]
    Unsupported,
    Struct(DynStruct),
    Enum(DynEnum),
}

impl Clone for Component {
    fn clone(&self) -> Self {
        match self {
            Component::Unsupported => Self::Unsupported,
            Component::Struct(s) => Self::Struct(DynStruct(s.0.clone_dynamic())),
            Component::Enum(s) => Self::Enum(DynEnum(s.0.clone_dynamic())),
        }
    }
}

// TODO: Change this to a derive when DynamicEnum derives Debug
impl Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "None"),
            Self::Struct(arg0) => f.debug_tuple("Struct").field(arg0).finish(),
            Self::Enum(_arg0) => write!(f, "DynamicEnum doesn't implement debug atm"),
        }
    }
}
