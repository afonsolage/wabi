use bevy::{
    ecs::component::ComponentInfo,
    prelude::{AppTypeRegistry, ReflectComponent, World},
};

use smallvec::SmallVec;
use wabi_runtime_api::mod_api::{
    ecs::{Component, Entity},
    query::{Filter, Query, QueryFetch, QueryFetchItem},
};

fn get_component_info<'w>(world: &'w World, name: &str) -> &'w ComponentInfo {
    world
        .components()
        .iter()
        .find(|c| c.name() == name)
        .expect("Should exists")
}

// TODO: Add error handling, since there is no point in panicking when running commands from wasm modules.
pub(crate) fn dynamic_query(world: &World, query: Query) -> QueryFetch {
    let registry_arc = world.resource::<AppTypeRegistry>();

    let with = query
        .filters
        .iter()
        .filter_map(|f| match f {
            Filter::With(name) => Some(get_component_info(world, name)),
            _ => None,
        })
        .collect::<SmallVec<[_; 8]>>();

    let without = query
        .filters
        .iter()
        .filter_map(|f| match f {
            Filter::Without(name) => Some(get_component_info(world, name)),
            _ => None,
        })
        .collect::<SmallVec<[_; 8]>>();

    let components = query
        .components
        .iter()
        .map(|name| get_component_info(world, name))
        .collect::<SmallVec<[_; 8]>>();

    let entities = world
        .archetypes()
        .iter()
        .filter_map(|arch| {
            if with.iter().all(|c| arch.contains(c.id()))
                && without.iter().all(|c| !arch.contains(c.id()))
                && components.iter().all(|c| arch.contains(c.id()))
            {
                Some(arch.entities())
            } else {
                None
            }
        })
        .flatten();

    let registry_guard = registry_arc.internal.read();

    let items = entities
        .map(|entity| QueryFetchItem {
            entity: Entity {
                id: entity.id(),
                generation: entity.generation(),
            },
            components: components
                .iter()
                .map(|component| {
                    let reflect_component = {
                        registry_guard
                            .get(component.type_id().unwrap())
                            .unwrap()
                            .data::<ReflectComponent>()
                            .unwrap()
                    };

                    Component::from(reflect_component.reflect(world, *entity).unwrap())
                })
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    QueryFetch { items }
}
