use bevy::{
    ecs::component::ComponentInfo,
    prelude::{AppTypeRegistry, ReflectComponent, World},
};
use wabi_runtime_api::mod_api::query::{Filter, Query};

fn get_component_info<'w>(world: &'w World, name: &str) -> Option<&'w ComponentInfo> {
    world.components().iter().find(|c| c.name() == name)
}

pub(crate) fn dynamic_query(world: &World, query: Query) {
    let registry_arc = world.resource::<AppTypeRegistry>();

    let with = query
        .filters
        .iter()
        .filter_map(|f| match f {
            Filter::With(name) => get_component_info(world, name),
            _ => None,
        })
        .collect::<Vec<_>>();

    let without = query
        .filters
        .iter()
        .filter_map(|f| match f {
            Filter::Without(name) => get_component_info(world, name),
            _ => None,
        })
        .collect::<Vec<_>>();

    let components = query
        .components
        .iter()
        .filter_map(|name| get_component_info(world, name))
        .collect::<Vec<_>>();

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

    let entity_components = entities
        .map(|entity| {
            (
                entity,
                components
                    .iter()
                    .map(|component| {
                        let reflect_component = {
                            registry_guard
                                .get(component.type_id().unwrap())
                                .unwrap()
                                .data::<ReflectComponent>()
                                .unwrap()
                        };

                        reflect_component.reflect(world, *entity).unwrap()
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    println!("Entity components: {:?}", entity_components);
}
