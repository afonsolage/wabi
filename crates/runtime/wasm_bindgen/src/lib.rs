use std::{cell::UnsafeCell, collections::HashMap};

use bevy_log::{error, info};
use js_sys::{
    Function, Reflect, Uint8Array,
    WebAssembly::{self, Memory},
};
use wabi_api::{
    InstanceState, WabiInstancePlatform, WabiRuntimePlatform, WABI_ALLOCATOR, WABI_ENTRY_POINT,
    WABI_MOODULE_NAME, WABI_PROCESS_ACTION,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};

fn get_runtime_data() -> &'static mut RuntimeData {
    // WebAssembly is single thread.
    thread_local! {
        static RUNTIME_DATA: UnsafeCell<RuntimeData>   = UnsafeCell::new(Default::default());
    }
    RUNTIME_DATA.with(|data|
        // SAFETY: WebAssembly only runs on single thread.
        unsafe {
            &mut *data.get()
        })
}

pub struct ModInstance {
    id: u32,

    alloc: Function,
    entry_point: Function,
    memory: WebAssembly::Memory,

    buffer: Vec<u8>,
    buffer_offset: u32,
}

impl WabiInstancePlatform for ModInstance {
    fn run_alloc(&mut self) {
        self.buffer_offset = self
            .alloc
            .call1(&JsValue::undefined(), &JsValue::from(self.id))
            .unwrap()
            .as_f64()
            .unwrap() as u32;
    }

    fn run_main(&mut self) {
        self.entry_point.call0(&JsValue::undefined()).unwrap();
    }

    fn read_buffer(&mut self, len: u32) -> &[u8] {
        self.buffer.resize(len as usize, 0);

        Uint8Array::new_with_byte_offset_and_length(&self.memory.buffer(), self.buffer_offset, len)
            .copy_to(&mut self.buffer);

        &self.buffer
    }
}

impl ModInstance {
    pub fn new(id: u32, instance: WebAssembly::Instance) -> Self {
        let memory = Reflect::get(&instance.exports(), &"memory".into())
            .unwrap()
            .dyn_into::<Memory>()
            .unwrap();

        let alloc = Reflect::get(&instance.exports(), &WABI_ALLOCATOR.into())
            .unwrap()
            .dyn_into::<Function>()
            .unwrap();

        let entry_point = Reflect::get(&instance.exports(), &WABI_ENTRY_POINT.into())
            .unwrap()
            .dyn_into::<Function>()
            .unwrap();

        Self {
            id,
            alloc,
            entry_point,
            memory,
            buffer: Default::default(),
            buffer_offset: 0,
        }
    }
}

pub struct RuntimeData {
    pub process_action: fn(u32, u32, u8),
    pub modules: HashMap<u32, InstanceState<ModInstance>>,
}

impl Default for RuntimeData {
    fn default() -> Self {
        Self {
            process_action: |_, _, _| (),
            modules: Default::default(),
        }
    }
}

pub struct WasmRuntime;

impl WabiRuntimePlatform for WasmRuntime {
    type ModuleInstance = ModInstance;

    fn new(process_action: fn(u32, u32, u8)) -> Self {
        // TODO: Find a better and reliable way of inject imports
        js_sys::eval(
            format!(
                r#"import('./wabi.js').then(m => {{
                    let imports = m.getImports();
                    imports.{} = {{
                        {}: m.{},
                    }};
                    window.wabi_imports = imports;
                }});"#,
                WABI_MOODULE_NAME, WABI_PROCESS_ACTION, WABI_PROCESS_ACTION
            )
            .as_str(),
        )
        .unwrap();

        get_runtime_data().process_action = process_action;

        Self
    }

    fn load_module(&mut self, id: u32, buffer: &[u8]) {
        let buffer = Vec::from(buffer);

        let window = web_sys::window().unwrap();
        let imports = window.get("wabi_imports").unwrap();

        info!("Imports: {:?}", imports);

        spawn_local(async move {
            let result = JsFuture::from(WebAssembly::instantiate_buffer(&buffer, &imports)).await;

            let result = match result {
                Ok(r) => r,
                Err(err) => {
                    error!("Failed to instantiate module: {:?}", err);
                    return;
                }
            };

            let instance = Reflect::get(&result, &"instance".into())
                .unwrap()
                .dyn_into::<WebAssembly::Instance>()
                .unwrap();

            let runtime_data = get_runtime_data();

            runtime_data
                .modules
                .insert(id, InstanceState::Idle(ModInstance::new(id, instance)));
        });
    }

    fn get_instance(&mut self, id: u32) -> Option<&mut Self::ModuleInstance> {
        if let InstanceState::Idle(instance) = get_runtime_data().modules.get_mut(&id).unwrap() {
            Some(instance)
        } else {
            None
        }
    }

    fn start_running_instance(&mut self, id: u32) -> Self::ModuleInstance {
        let runtime_data = get_runtime_data();
        let instance = runtime_data.modules.insert(id, InstanceState::Running);
        instance.expect("Can run only existing instances").take()
    }

    fn finish_running_instance(&mut self, id: u32, instance: Self::ModuleInstance) {
        let runtime_data = get_runtime_data();
        let instance = runtime_data
            .modules
            .insert(id, InstanceState::Idle(instance));

        assert!(
            instance.unwrap().is_running(),
            "Cannot finish an instance which wasn't previous running"
        );
    }
}

// pub fn deserialize(buffer: &[u8]) {
//     let registry = TypeRegistry::default();
//     let deserializer = ReflectDeserializer::new(&registry);
//     let mut bin_code =
//         bincode::Deserializer::from_slice(buffer, bincode::config::DefaultOptions::default());
//     let value = deserializer.deserialize(&mut bin_code).unwrap();
// }

pub mod wabi {
    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn __wabi_process_action(id: u32, len: usize, action: u8) {
        (super::get_runtime_data().process_action)(id, len as u32, action);
    }
}
