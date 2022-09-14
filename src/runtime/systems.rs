use bevy::prelude::{AssetEvent, Assets, EventReader, Res, ResMut, World};

use crate::asset::WasmAsset;

use super::WabiRuntime;

pub(super) fn run_modules(world: &mut World) {
    world.resource_scope::<WabiRuntime, _>(|world, mut runtime| {
        runtime.run_all(world);
    });
}

pub(crate) fn load_wasm_modules(
    mut assets_events: EventReader<AssetEvent<WasmAsset>>,
    mut runtime: ResMut<WabiRuntime>,
    wams: Res<Assets<WasmAsset>>,
) {
    for evt in assets_events.iter() {
        if let AssetEvent::Created { handle } = evt {
            let asset = wams.get(handle).expect("Asset should be loaded");
            runtime.load_module(&asset.name, &asset.buffer);
        }
    }
}
