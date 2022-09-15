use bevy_reflect::{FromReflect, Reflect};

use crate::ecs::{Component, Entity};

#[derive(Reflect, Default, Debug, FromReflect, Clone)]
pub enum Filter {
    #[default]
    None,
    With(String),
    Without(String),
}

#[derive(Reflect, FromReflect, Default, Debug)]
pub struct Query {
    pub components: Vec<String>,
    pub filters: Vec<Filter>,
}

#[derive(Reflect, FromReflect, Default, Debug)]
pub struct QueryFetchItem {
    pub entity: Entity,
    pub components: Vec<Component>,
}

#[derive(Reflect, FromReflect, Default, Debug)]
pub struct QueryFetch {
    pub items: Vec<QueryFetchItem>,
}
