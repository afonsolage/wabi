use asset::WasmAsset;
use bevy::{
    asset::AssetServerSettings,
    log::{Level, LogSettings},
    prelude::*,
};

use runtime::WabiRuntime;

mod asset;
mod runtime;

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..Default::default()
        })
        .insert_resource(LogSettings {
            filter: "wabi=trace,error".to_string(),
            level: Level::TRACE,
        })
        .add_plugins(DefaultPlugins)
        .add_asset::<WasmAsset>()
        .init_asset_loader::<WasmAsset>()
        .init_resource::<WabiRuntime>()
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
    mut runtime: ResMut<WabiRuntime>,
    mut local: Local<Stage>,
) {
    if let Stage::Done = *local {
        return;
    }

    match *local {
        Stage::AssetLoading => {
            if let Some(wasm) = wasms.get(&*handle) {
                runtime.load_module("dummy".to_string(), &wasm.0);
                *local = Stage::ModLoading;
            }
        }
        Stage::ModLoading => {
            runtime.run("dummy");
            *local = Stage::Done;
        }
        Stage::Done => (),
    }
}
