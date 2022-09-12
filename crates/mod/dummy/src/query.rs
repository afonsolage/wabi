use bevy_reflect::FromReflect;
use wabi_mod_api::{
    query::{Query, QueryFetch, Select},
    Action,
};

use crate::{io::send_action, wabi::unwrap};

pub fn query(components: &[&'static str]) -> QueryFetch {
    let query = Query {
        selects: components
            .iter()
            .map(|c| Select::ReadOnly(c.to_string()))
            .collect(),
        filters: vec![],
    };

    let result = unwrap!(send_action(&query, Action::QUERY));
    unwrap!(QueryFetch::from_reflect(result.as_ref()))
}