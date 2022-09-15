use bevy_reflect::FromReflect;
use wabi_mod_api::{
    query::{Query, QueryFetch, Filter},
    Action,
};

use crate::{io::send_action, wabi::unwrap};

pub fn query(components: &[&'static str], filters: &[Filter]) -> QueryFetch {
    let query = Query {
        components: components.iter().map(ToString::to_string).collect(),
        filters: filters.into(),
    };

    let result = unwrap!(send_action(&query, Action::QUERY));
    unwrap!(QueryFetch::from_reflect(result.as_ref()))
}
