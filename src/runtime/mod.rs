use std::{cell::RefCell, error::Error, fmt::Display};

use bevy::{
    prelude::{error, trace, CoreStage, IntoExclusiveSystem, Plugin, Resource, World},
    utils::HashMap,
};
use bevy_reflect::TypeRegistry;
use smallvec::SmallVec;
use wabi_runtime_api::{
    mod_api::registry::create_type_registry, WabiInstancePlatform, WabiRuntimePlatform,
};

mod context;
pub mod systems;

#[cfg(not(target_arch = "wasm32"))]
pub type Platform = wabi_wasmtime::WasmtimeRuntime;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub type Platform = wabi_wasm_bindgen::WasmRuntime;
#[cfg(all(target_arch = "wasm32", not(target_os = "unknown")))]
compile_error!("Only target platform wasm32-unknown-unknown is supported atm.");

thread_local! {
    /// Each thread can only run one wasm module at a time, since all modules functions are blocking
    static RUNNING_CONTEXT: RefCell<context::Context> = Default::default();
}

type WabiInstance = <Platform as WabiRuntimePlatform>::ModuleInstance;

pub(super) struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<WabiRuntime>()
            .add_system(systems::run_modules.exclusive_system())
            .add_system_to_stage(CoreStage::PreUpdate, systems::load_wasm_modules);
    }
}

#[derive(Debug)]
pub enum WabiError {
    ModuleNotFound(String),
}

impl Display for WabiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WabiError::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
        }
    }
}

impl Error for WabiError {}

#[derive(Resource)]
pub struct WabiRuntime<P: WabiRuntimePlatform = Platform> {
    inner: P,
    instances_name_map: HashMap<String, u32>,
    last_id: u32,
    type_registry: TypeRegistry,
}

impl WabiRuntime {
    pub fn get_module_id(&self, name: &str) -> Result<u32, WabiError> {
        self.instances_name_map
            .get(name)
            .copied()
            .ok_or_else(|| WabiError::ModuleNotFound(name.to_string()))
    }

    pub fn load_module(&mut self, name: &str, buffer: &[u8]) {
        // TODO: Convert this to a soft error later on
        assert!(
            self.get_module_id(name).is_err(),
            "Can't have two modules with same name"
        );

        self.last_id += 1;
        self.inner.load_module(self.last_id, buffer);
        self.instances_name_map
            .insert(name.to_string(), self.last_id);
    }

    pub fn run_all(&mut self, world: &mut World) {
        let modules = self
            .instances_name_map
            .keys()
            .cloned()
            .collect::<SmallVec<[_; 8]>>();

        for name in modules {
            if let Err(err) = self.run(world, &name) {
                error!("Failed to run module {}. Error: {}", &name, err);
            }
        }
    }

    pub fn run(&mut self, world: &mut World, name: &str) -> Result<(), WabiError> {
        let id = self.get_module_id(name)?;

        trace!("Running module {}", name);

        // let begin = Instant::now();
        let mut instance = self.inner.start_running_instance(id);

        // TODO: Find a better place for this
        instance.run_alloc();

        // let alloc = Instant::now();
        RUNNING_CONTEXT.with(|cell| {
            cell.borrow_mut()
                .setup(world, &mut instance, &self.type_registry)
        });

        instance.run_main();

        // let finished = Instant::now();

        // trace!(
        //     "alloc: {}us, finished: {}us",
        //     (alloc - begin).as_micros(),
        //     (finished - alloc).as_micros()
        // );

        RUNNING_CONTEXT.with(|cell| {
            cell.borrow_mut().teardown();
        });

        self.inner.finish_running_instance(id, instance);

        Ok(())
    }

    fn process_action(id: u32, len: u32, action: u8) -> u32 {
        RUNNING_CONTEXT.with(|cell| cell.borrow_mut().process_action(id, len, action.into()))
    }
}

impl Default for WabiRuntime {
    fn default() -> Self {
        Self {
            inner: Platform::new(Self::process_action),
            instances_name_map: Default::default(),
            last_id: 0,
            type_registry: create_type_registry(),
        }
    }
}
