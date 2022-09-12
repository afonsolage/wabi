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
        match reflect.reflect_ref() {
            ReflectRef::Struct(v) => Some(Self(v.clone_dynamic())),
            ReflectRef::TupleStruct(v) => Some(Self(
                v.field(0)
                    .expect("Should have a field")
                    .downcast_ref::<DynamicStruct>()
                    .unwrap()
                    .clone_dynamic(),
            )),
            _ => None,
        }
    }
}

#[derive(Reflect, Default)]
pub struct DynEnum(DynamicEnum);

impl FromReflect for DynEnum {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        match reflect.reflect_ref() {
            ReflectRef::Enum(v) => Some(Self(v.clone_dynamic())),
            ReflectRef::TupleStruct(v) => Some(Self(
                v.field(0)
                    .expect("Should have a field")
                    .downcast_ref::<DynamicEnum>()
                    .unwrap()
                    .clone_dynamic(),
            )),
            _ => None,
        }
    }
}

// TODO: Change this to a derive when DynamicEnum derives Debug
impl Debug for DynEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DynEnum (DynamicEnum doesn't support debug yet)")
            .finish()
    }
}

#[derive(Reflect, FromReflect, Default, Debug)]
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
