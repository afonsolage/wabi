use std::collections::HashMap;

use bevy::prelude::warn;
use wabi_api::WabiRuntime;
use wasmtime::*;

pub struct WasmtimeRuntime {
    engine: Engine,
    store: Store<()>,

    modules: HashMap<String, Module>,
    instances: HashMap<String, Instance>,
}

// Implement __wbindgen_throw mock
// Implement __wabi_process_action

impl WabiRuntime for WasmtimeRuntime {
    fn new() -> Self {
        let engine = Engine::default();
        let store = Store::new(&engine, ());

        Self {
            engine,
            store,
            modules: Default::default(),
            instances: Default::default(),
        }
    }

    fn load_mod(&mut self, name: String, buffer: &[u8]) {
        let module = Module::from_binary(&self.engine, buffer).unwrap();

        for import in module.imports() {
            warn!("Import needed: {:?}", import);
        }

        let instance = Instance::new(&mut self.store, &module, &[]).unwrap();

        self.modules.insert(name.clone(), module);
        self.instances.insert(name, instance);
    }

    fn run(&mut self, name: &str) -> i32 {
        let func = self
            .instances
            .get(name)
            .unwrap()
            .get_typed_func::<(), i32, _>(&mut self.store, "run")
            .unwrap();

        func.call(&mut self.store, ()).unwrap()
    }

    fn add_export<F>(&mut self, name: String, f: F)
    where
        F: Fn(),
    {
        todo!()
    }
}

pub fn run() -> i32 {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());

    let module = Module::from_file(&engine, "web/dummy.wasm").unwrap();

    let instance = Instance::new(&mut store, &module, &[]).unwrap();

    let func = instance
        .get_typed_func::<(), i32, _>(&mut store, "run")
        .unwrap();
    func.call(&mut store, ()).unwrap()
}
