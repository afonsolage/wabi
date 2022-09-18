use std::marker::PhantomData;

use wabi_macros::proxy;

struct Proxy<T>(PhantomData<T>);

proxy!(bevy::prelude::Vec3);

fn main() {
    
}
