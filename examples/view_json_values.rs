use bevy_reflect::{serde::ReflectSerializer, Reflect};
use wabi_runtime_api::mod_api::{
    ecs::Component,
    log::LogMessage,
    query::{QueryFetch, QueryFetchItem},
    registry::create_type_registry,
};

fn main() {
    let log = LogMessage::default();

    let component = Component::from(log.as_reflect());

    let log = QueryFetch {
        items: vec![QueryFetchItem {
            entity: Default::default(),
            components: vec![component],
        }],
    };

    println(log);
}

fn println<T: Reflect>(reflect: T) {
    let registry = create_type_registry();

    let mut buffer = vec![];

    rmp_serde::encode::write(
        &mut buffer,
        &ReflectSerializer::new(reflect.as_reflect(), &registry),
    )
    .unwrap();

    let value: rmpv::Value = rmp_serde::from_slice(&buffer).unwrap();
    let json = serde_json::to_string_pretty(&value).unwrap();
    println!("{}", json);
}
