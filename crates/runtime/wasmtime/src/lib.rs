use std::collections::HashMap;

use bevy::prelude::{error, warn};
use wabi_api::{WabiInstancePlatform, WabiRuntimePlatform};
use wasmtime::*;

pub struct WasmtimeInstance {
    init: TypedFunc<u32, u32>,
    main: TypedFunc<(), ()>,

    store: Store<()>,
    memory: Memory,
}

impl WabiInstancePlatform for WasmtimeInstance {
    fn run_alloc(&mut self) {
        todo!()
    }

    fn run_main(&mut self) {
        todo!()
    }

    fn read_buffer(&mut self, len: u32) -> &[u8] {
        todo!()
    }
}

pub struct WasmtimeRuntime {
    engine: Engine,
    linker: Linker<()>,

    instances: HashMap<u32, WasmtimeInstance>,
}

// Implement __wbindgen_throw mock
// Implement __wabi_process_action

impl WabiRuntimePlatform for WasmtimeRuntime {
    type ModuleInstance = WasmtimeInstance;

    fn new(process_action: fn(u32, u32, u8)) -> Self {
        let engine = Engine::default();

        let mut linker = Linker::new(&engine);

        linker
            .func_wrap(
                "wbg",
                "__wbindgen_throw",
                |_caller: Caller<'_, ()>, _ptr: i32, _len: i32| {
                    error!("Mod is trying to throw an error, but it's not implemented yet");
                },
            )
            .unwrap();

        linker
            .func_wrap(
                "wabi",
                "__wabi_process_action",
                move |_caller: Caller<'_, ()>, id: u32, len: u32, action: u32| {
                    (process_action)(id, len, action as u8);
                },
            )
            .unwrap();

        Self {
            engine,
            linker,
            instances: Default::default(),
        }
    }

    fn load_module(&mut self, id: u32, buffer: &[u8]) {
        let module = Module::from_binary(&self.engine, buffer).unwrap();
        let mut store = Store::new(&self.engine, ());

        let instance = self.linker.instantiate(&mut store, &module).unwrap();

        let init = instance
            .get_func(&mut store, "__wabi_init")
            .unwrap()
            .typed(&mut store)
            .unwrap();

        let main = instance
            .get_func(&mut store, "__wabi_main")
            .unwrap()
            .typed(&mut store)
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        self.instances.insert(
            id,
            WasmtimeInstance {
                init,
                main,
                memory,
                store,
            },
        );
    }

    fn get_instance(&mut self, id: u32) -> Option<&mut Self::ModuleInstance> {
        self.instances.get_mut(&id)
    }

    fn start_running_instance(&mut self, id: u32) -> Self::ModuleInstance {
        todo!()
    }

    fn finish_running_instance(&mut self, id: u32, instance: Self::ModuleInstance) {
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
