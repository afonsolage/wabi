use std::{cell::UnsafeCell, collections::HashMap};

use bevy_log::{error, info};
use bevy_wabi_api::WabiRuntime;
use js_sys::{
    Function, Reflect, Uint8Array,
    WebAssembly::{self, Memory},
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

struct ModInstance {
    id: u32,

    instance: WebAssembly::Instance,
    init: Function,
    main: Function,
    memory: WebAssembly::Memory,

    buffer: Vec<u8>,
    buffer_offset: u32,
}

impl ModInstance {
    pub fn new(id: u32, instance: WebAssembly::Instance) -> Self {
        let memory = Reflect::get(&instance.exports(), &"memory".into())
            .unwrap()
            .dyn_into::<Memory>()
            .unwrap();

        let allocator = Reflect::get(&instance.exports(), &"__wabi_init".into())
            .unwrap()
            .dyn_into::<Function>()
            .unwrap();

        let main = Reflect::get(&instance.exports(), &"__wabi_main".into())
            .unwrap()
            .dyn_into::<Function>()
            .unwrap();

        Self {
            id,
            instance,
            init: allocator,
            main,
            memory,

            buffer: Default::default(),
            buffer_offset: Default::default(),
        }
    }

    pub fn run_main(&mut self) {
        self.buffer_offset = self
            .init
            .call1(&JsValue::undefined(), &JsValue::from(self.id))
            .unwrap()
            .as_f64()
            .unwrap() as u32;

        self.main.call0(&JsValue::undefined()).unwrap();
    }

    pub fn read_buffer(&mut self, len: usize) {
        let array = Uint8Array::new_with_byte_offset_and_length(
            &self.memory.buffer(),
            self.buffer_offset as u32,
            len as u32,
        );

        self.buffer.resize(len as usize, 0);
        array.copy_to(&mut self.buffer);
    }
}

#[derive(Default)]
struct RuntimeData {
    last_instance_id: u32,
    pub modules: HashMap<String, ModInstance>,
}

impl RuntimeData {
    pub fn gen_instance_id(&mut self) -> u32 {
        self.last_instance_id += 1;
        self.last_instance_id
    }
}

pub struct WasmRuntime {}

impl WabiRuntime for WasmRuntime {
    fn new() -> Self {
        // TODO: Find a better and reliable way of inject imports
        js_sys::eval(
            r#"import('./wabi.js').then(m => {
            let imports = m.getImports();
            imports.wabi = {
                __wabi_process_action: m.__wabi_process_action,
            };
            window.wabi_imports = imports;
        });"#,
        )
        .unwrap();

        Self {}
    }

    fn load_mod(&mut self, name: String, buffer: &[u8]) {
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
            let id = runtime_data.gen_instance_id();

            runtime_data
                .modules
                .insert(name, ModInstance::new(id, instance));
        });
    }

    fn run(&mut self, name: &str) -> i32 {
        let runtime_data = get_runtime_data();
        let instance = runtime_data.modules.get_mut(name).unwrap();
        instance.run_main();
        1
    }

    fn add_export<F>(&mut self, _name: String, _f: F)
    where
        F: Fn(),
    {
        todo!()
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
    use bevy_log::info;
    use bevy_wabi_api::process_action;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub fn __wabi_process_action(id: u32, len: usize, action: u8) {
        info!("receiving data ({} len) from id: {}", len, id);
        let runtime_data = super::get_runtime_data();

        let module = runtime_data
            .modules
            .values_mut()
            .find(|m| m.id == id)
            .unwrap();

        module.read_buffer(len);
        process_action(&module.buffer, action.into());
    }
}
