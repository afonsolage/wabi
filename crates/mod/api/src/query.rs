use bevy_reflect::{FromReflect, Reflect};

use crate::ecs::{Component, Entity};

#[derive(Reflect, Default, Debug, FromReflect)]
pub enum Select {
    #[default]
    None,
    Entity,
    ReadOnly(String),
    Mutable(String),
}

#[derive(Reflect, Default, Debug, FromReflect)]
pub enum Filter {
    #[default]
    None,
    Changed(String),
    With(String),
    Without(String),
}

#[derive(Reflect, Default, Debug)]
pub struct Query {
    selects: Vec<Select>,
    filters: Vec<Filter>,
}

#[derive(Reflect, FromReflect, Default, Debug, Clone)]
pub enum QueryFetchItem {
    #[default]
    None,
    Entity(Entity),
    ReadOnly(Component),
    Mutable(Component),
}

#[derive(Reflect, Default, Debug)]
pub struct QueryFetch {
    pub items: Vec<QueryFetchItem>,
}
