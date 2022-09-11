use bevy_reflect::{serde::ReflectSerializer, FromReflect, Reflect};
use wabi_runtime_api::mod_api::{
    ecs::{Component, DynStruct},
    log::LogMessage,
    query::{QueryFetch, QueryFetchItem},
    registry::create_type_registry,
};

fn main() {
    let log = LogMessage::default();

    let component = Component::Struct(DynStruct::from_reflect(log.as_reflect()).unwrap());

    let log = QueryFetch {
        items: vec![QueryFetchItem::ReadOnly(component)],
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
