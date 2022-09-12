use bevy_reflect::{
    erased_serde::__private::serde::de::DeserializeSeed,
    serde::{ReflectDeserializer, ReflectSerializer},
    FromReflect, Reflect,
};
use wabi_runtime_api::mod_api::{
    ecs::{Component, DynEnum, DynStruct},
    log::LogMessage,
    query::{QueryFetch, QueryFetchItem},
    registry::create_type_registry,
};

fn main() {
    let dummy = LogMessage::default();
    let maybe = Some("Nested enum".to_string());

    let component_struct = Component::Struct(DynStruct::from_reflect(dummy.as_reflect()).unwrap());
    let simple_enum = Component::Enum(DynEnum::from_reflect(maybe.as_reflect()).unwrap());
    let data = QueryFetch {
        items: vec![
            QueryFetchItem::ReadOnly(component_struct),
            QueryFetchItem::Mutable(simple_enum),
        ],
        // items: vec![],
    };

    let type_registry = create_type_registry();

    let value = data.as_reflect();

    let encoded =
        rmp_serde::encode::to_vec(&ReflectSerializer::new(value, &type_registry)).unwrap();

    let json =
        serde_json::to_string_pretty(&ReflectSerializer::new(value, &type_registry)).unwrap();

    let reflect_deserializer = ReflectDeserializer::new(&type_registry);
    let mut deserializer = rmp_serde::decode::Deserializer::from_read_ref(&encoded);
    let decoded = reflect_deserializer.deserialize(&mut deserializer).unwrap();

    let reflect_deserializer = ReflectDeserializer::new(&type_registry);
    let mut deserializer = serde_json::de::Deserializer::from_str(&json);
    let json_decoded = reflect_deserializer.deserialize(&mut deserializer).unwrap();

    println!("Before: {:?}", value);
    println!("Decoded: {:?}", decoded);

    println!("Before Json: {}", json);
    println!("Decoded Json: {:?}", json_decoded);

    let query = QueryFetch::from_reflect(&*decoded).unwrap();
    println!("Query: {:?}", query);
}
