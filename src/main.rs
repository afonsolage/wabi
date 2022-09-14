use asset::WasmAsset;
use bevy::{
    asset::AssetServerSettings,
    log::{Level, LogSettings},
    prelude::*,
};

use reflect_query::dynamic_query;
use runtime::WabiRuntime;
use wabi_runtime_api::mod_api::query::{self};

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
        .add_asset::<WasmAsset>()
        .init_asset_loader::<WasmAsset>()
        .init_resource::<WabiRuntime>()
        .add_startup_system_to_stage(StartupStage::PreStartup, pre_startup)
        .add_startup_system(scene_setup)
        .add_system(startup)
        .add_system(test.exclusive_system())
        .run();
}

#[derive(Resource, Component, Reflect, Debug, Default, Deref, DerefMut)]
struct WasmHandler(pub Handle<WasmAsset>);

fn pre_startup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(WasmHandler(asset_server.load("mods/dummy.wasm")));
}

#[derive(Default)]
enum Stage {
    #[default]
    AssetLoading,
    ModLoading,
    Done,
}

fn startup(
    handle: Res<WasmHandler>,
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

fn test(world: &mut World) {
    let target = "bevy_transform::components::transform::Transform";

    let query = query::Query {
        components: vec![target.to_string()],
        filters: vec![],
    };

    dynamic_query(world, query);

    let mut state = world.query::<(Entity, &Transform)>();
    for (entity, t) in state.iter(world) {
        println!("Entity: {:?}, T: {:?}", entity, t);
    }

    std::process::exit(0);
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
