mod actor;
mod ai;
mod camera;
mod combat;
mod console;
mod game_state;
mod hud;
mod item;
mod menu;
mod rendering;
mod scripting;
mod weapon;
mod world;
use actor::ActorPlugin;
use ai::AIPlugin;
use bevy::prelude::*;
use camera::{CameraPlugin, Player, PlayerLightPlugin, update_camera_shake};
use combat::{update_blood_particles, update_damage_numbers, update_status_effects};
use console::*;
use game_state::{GamePlayEntity, GameState, GameStatePlugin};
use hud::{Toolbar, startup_ui, update_ui};
use item::ItemPlugin;
use menu::MenuPlugin;
use rendering::update_billboards;
use scripting::{CVarRegistry, ScriptingPlugin};
use weapon::WeaponPlugin;
use world::{Map, MapEditorPlugin, WorldPlugin};

// MapFile and MapData are now defined in map.rs

fn main() {
    // Get asset path from REPO_ROOT environment variable
    let asset_path = std::env::var("REPO_ROOT")
        .map(|repo_root| format!("{}/source/assets", repo_root))
        .unwrap();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::asset::AssetPlugin {
                    file_path: asset_path,
                    ..default()
                })
                .set(bevy::window::WindowPlugin {
                    primary_window: Some(bevy::window::Window {
                        title: "Fallgray".into(),
                        resolution: (1920, 1080).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, (log_startup, setup_ui_camera))
        .add_plugins(ScriptingPlugin)
        .add_plugins(GameStatePlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerLightPlugin)
        .add_plugins(AIPlugin)
        .add_plugins(ActorPlugin)
        .add_plugins(ItemPlugin)
        .add_plugins(ConsolePlugin {})
        .add_plugins(hud::ToolbarPlugin)
        .add_plugins(WeaponPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(MapEditorPlugin)
        // Main Menu systems
        // Playing state systems
        .add_systems(OnEnter(GameState::Playing), startup_ui)
        .add_systems(
            Update,
            (
                update_camera_shake.after(camera::update_camera_control_system),
                update_damage_numbers,
                update_blood_particles,
                update_status_effects,
                update_ui,
                update_billboards,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .run();
}

fn log_startup(current_state: Res<State<GameState>>) {
    info!("=== Game Starting ===");
    info!("Initial game state: {:?}", current_state.get());
}

/// Setup a persistent UI camera for menus
fn setup_ui_camera(mut commands: Commands) {
    info!("Spawning persistent UI camera for menus");
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
    ));
}
