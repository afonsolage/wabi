use std::collections::HashMap;

use bevy::prelude::error;
use wabi_api::{
    InstanceState, WabiInstancePlatform, WabiRuntimePlatform, WABI_ALLOCATOR, WABI_ENTRY_POINT,
    WABI_MOODULE_NAME, WABI_PROCESS_ACTION,
};
use wasmtime::*;

pub struct WasmtimeInstance {
    id: u32,

    init: TypedFunc<u32, u32>,
    main: TypedFunc<(), ()>,

    store: Store<()>,
    memory: Memory,

    buffer_offset: u32,
}

impl WabiInstancePlatform for WasmtimeInstance {
    fn run_alloc(&mut self) {
        self.buffer_offset = self.init.call(&mut self.store, self.id).unwrap();
    }

    fn run_main(&mut self) {
        self.main.call(&mut self.store, ()).unwrap();
    }

    fn read_buffer(&mut self, len: u32) -> &[u8] {
        let begin = self.buffer_offset as usize;
        let end = begin + len as usize;
        &self.memory.data(&mut self.store)[begin..end]
    }
}

pub struct WasmtimeRuntime {
    engine: Engine,
    linker: Linker<()>,

    instances: HashMap<u32, InstanceState<WasmtimeInstance>>,
}

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
                WABI_MOODULE_NAME,
                WABI_PROCESS_ACTION,
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
            .get_func(&mut store, WABI_ALLOCATOR)
            .unwrap()
            .typed(&mut store)
            .unwrap();

        let main = instance
            .get_func(&mut store, WABI_ENTRY_POINT)
            .unwrap()
            .typed(&mut store)
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        self.instances.insert(
            id,
            InstanceState::Idle(WasmtimeInstance {
                id,
                init,
                main,
                memory,
                store,
                buffer_offset: 0,
            }),
        );
    }

    fn get_instance(&mut self, id: u32) -> Option<&mut Self::ModuleInstance> {
        if let Some(InstanceState::Idle(instance)) = self.instances.get_mut(&id) {
            Some(instance)
        } else {
            None
        }
    }

    fn start_running_instance(&mut self, id: u32) -> Self::ModuleInstance {
        self.instances
            .insert(id, InstanceState::Running)
            .expect("Should exists")
            .take()
    }

    fn finish_running_instance(&mut self, id: u32, instance: Self::ModuleInstance) {
        let previous = self.instances.insert(id, InstanceState::Idle(instance));
        debug_assert!(previous
            .expect("Should have a previous state of Running")
            .is_running())
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
