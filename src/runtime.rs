use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use bevy::{
    prelude::{debug, error, info, trace, warn, Resource},
    reflect::{erased_serde::__private::serde::de::DeserializeSeed, serde::ReflectDeserializer},
    utils::HashMap,
};
use bevy_reflect::{serde::ReflectSerializer, FromReflect, Reflect, TypeRegistry};
use wabi_runtime_api::{
    mod_api::{
        ecs::{Component, DynStruct},
        log::LogMessage,
        query::{Query, QueryFetch, QueryFetchItem},
        registry::create_type_registry,
        Action,
    },
    WabiInstancePlatform, WabiRuntimePlatform,
};

#[cfg(not(target_arch = "wasm32"))]
pub type Platform = wabi_wasmtime::WasmtimeRuntime;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub type Platform = wabi_wasm_bindgen::WasmRuntime;
#[cfg(all(target_arch = "wasm32", not(target_os = "unknown")))]
compile_error!("Only target platform wasm32-unknown-unknown is supported atm.");

thread_local! {
    /// Each thread can only run one wasm module at a time, since all modules functions are blocking
    static RUNNING_CONTEXT: RefCell<Context> = Default::default();
}

type WabiInstance = <Platform as WabiRuntimePlatform>::ModuleInstance;

struct Context {
    instance: *mut WabiInstance,
    registry: Arc<Mutex<TypeRegistry>>,
}

impl Context {
    fn setup(&mut self, instance: &mut WabiInstance, registry: Arc<Mutex<TypeRegistry>>) {
        debug_assert!(self.instance.is_null());
        self.instance = instance;
        self.registry = registry;
    }

    fn tear_down(&mut self) {
        self.registry = Arc::default();
        self.instance = std::ptr::null_mut();
    }

    /// **This function should be called only on a callback from wasm module.**
    unsafe fn instance(&self) -> &'static mut WabiInstance {
        debug_assert!(!self.instance.is_null());
        &mut *self.instance
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            instance: std::ptr::null_mut(),
            registry: Default::default(),
        }
    }
}

#[derive(Resource)]
pub struct WabiRuntime<P: WabiRuntimePlatform = Platform> {
    inner: P,
    type_registry: Arc<Mutex<TypeRegistry>>,
    instances_name_map: HashMap<String, u32>,
    last_id: u32,
}

impl Default for WabiRuntime {
    fn default() -> Self {
        Self {
            inner: Platform::new(Self::receive_action),
            type_registry: Arc::new(Mutex::new(create_type_registry())),
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

        // let begin = Instant::now();
        let mut instance = self.inner.start_running_instance(id);

        // TODO: Find a better place for this
        instance.run_alloc();

        // let alloc = Instant::now();
        RUNNING_CONTEXT.with(|cell| {
            cell.borrow_mut()
                .setup(&mut instance, self.type_registry.clone())
        });

        instance.run_main();

        // let finished = Instant::now();

        // trace!(
        //     "alloc: {}us, finished: {}us",
        //     (alloc - begin).as_micros(),
        //     (finished - alloc).as_micros()
        // );

        RUNNING_CONTEXT.with(|cell| {
            cell.borrow_mut().tear_down();
        });

        self.inner.finish_running_instance(id, instance);
    }

    fn send_response(data: Box<dyn Reflect>) -> u32 {
        let (registry, instance) = RUNNING_CONTEXT.with(|context| {
            let context = context.borrow_mut();
            (
                context.registry.clone(),
                // SAFETY: Receive action is called only by wasm, so there is a running instance
                // and also a valid instance pointer on thread local state.
                unsafe { context.instance() },
            )
        });

        let buffer = {
            let type_registry = registry.lock().unwrap();
            #[cfg(not(feature = "json"))]
            {
                rmp_serde::encode::to_vec(&ReflectSerializer::new(&*data, &type_registry)).unwrap()
            }
            #[cfg(feature = "json")]
            {
                serde_json::to_vec(&ReflectSerializer::new(&*data, &type_registry)).unwrap()
            }
        };

        instance.writer_buffer(&buffer);

        buffer.len() as u32
    }

    fn receive_action(id: u32, len: u32, action: u8) -> u32 {
        let (registry, instance) = RUNNING_CONTEXT.with(|context| {
            let context = context.borrow_mut();
            (
                context.registry.clone(),
                // SAFETY: Receive action is called only by wasm, so there is a running instance
                // and also a valid instance pointer on thread local state.
                unsafe { context.instance() },
            )
        });

        assert!(instance.id() == id);

        let value = {
            let type_registry = registry.lock().unwrap();
            let buffer = instance.read_buffer(len);

            let reflect_deserializer = ReflectDeserializer::new(&type_registry);

            let mut deserializer = {
                #[cfg(not(feature = "json"))]
                {
                    rmp_serde::Deserializer::from_read_ref(buffer)
                }

                #[cfg(feature = "json")]
                {
                    serde_json::Deserializer::from_slice(buffer)
                }
            };

            reflect_deserializer.deserialize(&mut deserializer).unwrap()
        };

        Self::process_action(instance, value, action.into())
    }

    fn process_action(_instance: &mut WabiInstance, data: Box<dyn Reflect>, action: Action) -> u32 {
        if action != Action::LOG {
            trace!("Received action: {:?}, data: {:?}", action, data);
        }

        let maybe_response = match action {
            Action::LOG => {
                let LogMessage { level, message } = LogMessage::from_reflect(&*data).unwrap();
                match level {
                    0 => trace!(message),
                    1 => debug!(message),
                    2 => info!(message),
                    3 => warn!(message),
                    4 => error!(message),
                    _ => error!("Invalid level received: {}. Message: ({})", level, message),
                };
                None
            }
            Action::QUERY => Some(Self::process_query(
                _instance,
                Query::from_reflect(&*data).unwrap(),
            )),
            Action::TEST => {
                debug!("Received: {:?}", data);
                None
            }
            Action::INVALID => {
                error!("Invalid action received.");
                None
            }
        };

        if let Some(response) = maybe_response {
            trace!("Sending response: {:?}, data: {:?}", action, response);
            Self::send_response(response)
        } else {
            0
        }
    }

    fn process_query(_instance: &mut WabiInstance, _query: Query) -> Box<dyn Reflect> {
        let dummy = LogMessage::default();

        let component = Component::Struct(DynStruct::from_reflect(dummy.as_reflect()).unwrap());
        let dummy = QueryFetch {
            items: vec![QueryFetchItem::ReadOnly(component)],
        };

        dummy.clone_value()
    }
}
