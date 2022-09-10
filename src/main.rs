use asset::WasmAsset;
use bevy::{asset::AssetServerSettings, prelude::*};

use wabi_api::WabiRuntime;

mod asset;

#[derive(Default, Deref, DerefMut)]
pub struct RuntimeImpl<T: WabiRuntime>(T);

impl<T: WabiRuntime> RuntimeImpl<T> {
    fn new() -> Self {
        Self(T::new())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type Runtime = RuntimeImpl<wabi_wasmtime::WasmtimeRuntime>;
#[cfg(target_arch = "wasm32")]
pub type Runtime = RuntimeImpl<wabi_wasm::WasmRuntime>;

fn main() {
    let mut app = App::new();

    app.insert_non_send_resource(Runtime::new());

    app.insert_resource(AssetServerSettings {
        watch_for_changes: true,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_asset::<WasmAsset>()
    .init_asset_loader::<WasmAsset>()
    .add_startup_system_to_stage(StartupStage::PreStartup, pre_startup)
    .add_system(startup)
    .run();
}

fn pre_startup(asset_server: Res<AssetServer>, mut commands: Commands) {
    let handle: Handle<WasmAsset> = asset_server.load("mods/dummy.wasm");

    commands.insert_resource(handle);
}
#[derive(Default)]
enum Stage {
    #[default]
    AssetLoading,
    ModLoading,
    Done,
}

fn startup(
    handle: Res<Handle<WasmAsset>>,
    wasms: Res<Assets<WasmAsset>>,
    mut runtime: NonSendMut<Runtime>,
    mut local: Local<Stage>,
) {
    if let Stage::Done = *local {
        return;
    }

    match *local {
        Stage::AssetLoading => {
            if let Some(wasm) = wasms.get(&*handle) {
                runtime.load_mod("dummy".to_string(), &wasm.0);
                *local = Stage::ModLoading;
            }
        }
        Stage::ModLoading => {
            let result = runtime.run("dummy");
            info!("Result: {}", result);
            *local = Stage::Done;
        }
        Stage::Done => (),
    }
}
