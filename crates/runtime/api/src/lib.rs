use std::ops::{Deref, DerefMut};

pub mod mod_api {
    pub use wabi_mod_api::*;
}

pub const WABI_MOODULE_NAME: &str = "wabi";
pub const WABI_ALLOCATOR: &str = "__wabi_alloc";
pub const WABI_ENTRY_POINT: &str = "__wabi_entry_point";
pub const WABI_PROCESS_ACTION: &str = "__wabi_process_action";

pub enum InstanceState<T: WabiInstancePlatform> {
    None,
    Idle(T),
    Running,
}

impl<T: WabiInstancePlatform> Default for InstanceState<T> {
    fn default() -> Self {
        Self::None
    }
}

impl<T: WabiInstancePlatform> Deref for InstanceState<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            InstanceState::None => {
                panic!("Cannot deref a non-existing module instance")
            }
            InstanceState::Idle(t) => t,
            InstanceState::Running => {
                panic!("Cannot deref a running module instance")
            }
        }
    }
}

impl<T: WabiInstancePlatform> DerefMut for InstanceState<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            InstanceState::None => {
                panic!("Cannot deref mut a non-existing module instance")
            }
            InstanceState::Idle(t) => t,
            InstanceState::Running => {
                panic!("Cannot deref mut a running module instance")
            }
        }
    }
}

impl<T: WabiInstancePlatform> InstanceState<T> {
    pub fn is_running(&self) -> bool {
        matches!(self, InstanceState::Running)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, InstanceState::Idle(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, InstanceState::None)
    }

    pub fn take(self) -> T {
        match self {
            InstanceState::None => {
                panic!("Cannot take a non-existing module instance")
            }
            InstanceState::Idle(t) => t,
            InstanceState::Running => {
                panic!("Cannot take a running module instance")
            }
        }
    }
}

pub trait WabiInstancePlatform {
    fn id(&self) -> u32;

    fn run_alloc(&mut self);
    fn run_main(&mut self);

    fn read_buffer(&mut self, len: u32) -> &[u8];
    fn writer_buffer(&mut self, buffer: &[u8]);
}

pub trait WabiRuntimePlatform {
    type ModuleInstance: WabiInstancePlatform;

    fn new(process_action: fn(u32, u32, u8) -> u32) -> Self;
    fn load_module(&mut self, id: u32, buffer: &[u8]);
    fn start_running_instance(&mut self, id: u32) -> Self::ModuleInstance;
    fn finish_running_instance(&mut self, id: u32, instance: Self::ModuleInstance);
    fn get_instance(&mut self, id: u32) -> Option<&mut Self::ModuleInstance>;
}
