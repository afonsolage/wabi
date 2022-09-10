use std::cell::UnsafeCell;

use bevy::{
    prelude::{debug, error, trace},
    reflect::{erased_serde::__private::serde::de::DeserializeSeed, serde::ReflectDeserializer},
    utils::HashMap,
};
use bevy_reflect::TypeRegistry;
use wabi_api::{create_type_registry, Action, WabiInstancePlatform, WabiRuntimePlatform};

#[cfg(not(target_arch = "wasm32"))]
pub type Platform = wabi_wasmtime::WasmtimeRuntime;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub type Platform = wabi_wasm_bindgen::WasmRuntime;
#[cfg(all(target_arch = "wasm32", not(target_os = "unknown")))]
compile_error!("Only target platform wasm32-unknown-unknown is supported atm.");

thread_local! {
    /// Each thread can only run one wasm module at a time, since all modules functions are blocking
    static RUNNING_INSTANCE: UnsafeCell<Option<*mut WabiInstance>> = Default::default();
}

type WabiInstance = <Platform as WabiRuntimePlatform>::ModuleInstance;

pub struct WabiRuntime<P: WabiRuntimePlatform = Platform> {
    inner: P,
    type_registry: TypeRegistry,
    instances_name_map: HashMap<String, u32>,
    last_id: u32,
}

impl Default for WabiRuntime {
    fn default() -> Self {
        Self {
            inner: Platform::new(Self::receive_action),
            type_registry: create_type_registry(),
            instances_name_map: Default::default(),
            last_id: 0,
        }
    }
}

impl WabiRuntime {
    pub fn get_module_id(&self, name: &str) -> Option<u32> {
        self.instances_name_map.get(name).copied()
    }

    pub fn load_module(&mut self, name: String, buffer: &[u8]) {
        // TODO: Convert this to a soft error later on
        assert!(
            self.get_module_id(&name).is_none(),
            "Can't have two modules with same name"
        );

        self.last_id += 1;
        self.inner.load_module(self.last_id, buffer);
        self.instances_name_map.insert(name, self.last_id);
    }

    pub fn run(&mut self, name: &str) {
        let id = self.get_module_id(name).unwrap();
        let mut instance = self.inner.start_running_instance(id);

        // TODO: Find a better place for this
        instance.run_alloc();

        RUNNING_INSTANCE.with(|cell| {
            // SAFETY: There is only one running instance per thread and all wasm
            // callbacks are blocking, so it's safe to assume that there will be no
            // concurrent modification between threads/tasks.
            unsafe {
                let previous = cell.get().replace(Some(&mut instance));
                debug_assert!(
                    previous.is_none(),
                    "All running instances should be cleared before finished"
                );
            }
        });

        instance.run_main();

        RUNNING_INSTANCE.with(|cell| {
            // SAFETY: There is only one running instance per thread and all wasm
            // callbacks are blocking, so it's safe to assume that there will be no
            // concurrent modification between threads/tasks.
            unsafe {
                cell.get().replace(None);
            }
        });

        self.inner.finish_running_instance(id, instance);
    }

    fn receive_action(_id: u32, len: u32, action: u8) {
        let instance = RUNNING_INSTANCE.with(|map| {
            // SAFETY: It's only possible to reach here through a wasm callback, so there is a running instance
            // and also a valid instance pointer on thread local state.
            unsafe {
                &mut *(map
                    .get()
                    .as_ref()
                    .expect("Should always be a valid pointer")
                    .expect("Should exists a pointer"))
            }
        });

        Self::process_action(instance.read_buffer(len), action.into());
    }

    fn process_action(buffer: &[u8], action: Action) {
        trace!("Received action {:?} ({})", action, buffer.len());

        let type_registry = create_type_registry();
        let reflect_deserializer = ReflectDeserializer::new(&type_registry);
        let mut deserializer = rmp_serde::Deserializer::from_read_ref(buffer);
        let value = reflect_deserializer.deserialize(&mut deserializer).unwrap();

        match action {
            Action::DEBUG => {
                let message = value.downcast_ref::<String>().unwrap();
                debug!("{}", message);
            }
            Action::INVALID => error!("Invalid action received."),
        }
    }
}
