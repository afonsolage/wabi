use bevy_reflect::{
    erased_serde::__private::serde::de::DeserializeSeed,
    serde::{ReflectSerializer, UntypedReflectDeserializer},
    FromReflect, Reflect,
};
use wabi_runtime_api::mod_api::{
    ecs::Component,
    log::LogMessage,
    query::{QueryFetch, QueryFetchItem},
    registry::create_type_registry,
};

fn main() {
    let dummy = LogMessage::default();
    let maybe = Some("Nested enum".to_string());

    let component_struct = Component::from(dummy.as_reflect());
    let simple_enum = Component::from(maybe.as_reflect());
    let data = QueryFetch {
        items: vec![QueryFetchItem {
            entity: Default::default(),
            components: vec![component_struct, simple_enum],
        }],
    };

    let type_registry = create_type_registry();

    let value = data.as_reflect();

    let encoded =
        rmp_serde::encode::to_vec(&ReflectSerializer::new(value, &type_registry)).unwrap();

    let json =
        serde_json::to_string_pretty(&ReflectSerializer::new(value, &type_registry)).unwrap();

    let reflect_deserializer = UntypedReflectDeserializer::new(&type_registry);
    let mut deserializer = rmp_serde::decode::Deserializer::from_read_ref(&encoded);
    let decoded = reflect_deserializer.deserialize(&mut deserializer).unwrap();

    let reflect_deserializer = UntypedReflectDeserializer::new(&type_registry);
    let mut deserializer = serde_json::de::Deserializer::from_str(&json);
    let json_decoded = reflect_deserializer.deserialize(&mut deserializer).unwrap();

    println!("Before: {:?}", value);
    println!("Decoded: {:?}", decoded);

    println!("Before Json: {}", json);
    println!("Decoded Json: {:?}", json_decoded);

    let query = QueryFetch::from_reflect(&*decoded).unwrap();
    println!("Query: {:?}", query);
}
