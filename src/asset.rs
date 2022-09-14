use bevy::{
    asset::{AssetLoader, LoadedAsset},
    // prelude::{AssetEvent, EventReader},
    reflect::{Reflect, TypeUuid},
};

#[derive(Debug, Default, TypeUuid, Reflect)]
#[uuid = "44ceeab1-69e2-4afb-b0e2-7d97d8d0bdda"]
pub struct WasmAsset {
    pub name: String,
    pub(crate) buffer: Vec<u8>,
}

impl AssetLoader for WasmAsset {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(WasmAsset {
                name: load_context
                    .path()
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                buffer: Vec::from(bytes),
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wasm"]
    }
}