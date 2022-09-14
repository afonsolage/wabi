use asset::WasmAsset;
use bevy::{
    asset::AssetServerSettings,
    log::{Level, LogSettings},
    prelude::*,
};

use runtime::RuntimePlugin;

mod asset;
mod reflect_query;
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
        .add_plugin(RuntimePlugin)
        .add_asset::<WasmAsset>()
        .init_asset_loader::<WasmAsset>()
        .add_startup_system_to_stage(StartupStage::PreStartup, pre_startup)
        .add_startup_system(scene_setup)
        .run();
}

#[derive(Resource, Component, Reflect, Debug, Default, Deref, DerefMut)]
struct WasmHandler(pub Handle<WasmAsset>);

fn pre_startup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(WasmHandler(asset_server.load("mods/impl.wasm")));
}

/// set up a simple 3D scene
fn scene_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
