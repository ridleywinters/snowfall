mod actor;
mod ai;
mod camera;
mod collision;
mod combat;
mod console;
mod game_state;
mod item;
mod logging;
mod map;
#[cfg(test)]
mod map_test;
mod menu;
mod rendering;
mod scripting;
mod texture_loader;
mod toolbar;
mod ui;
mod ui_styles;
mod weapon;
mod world;
use actor::ActorPlugin;
use actor::*;
use ai::AIPlugin;
use bevy::prelude::*;
use camera::{CameraPlugin, Player, PlayerLightPlugin, update_camera_shake};
use collision::check_circle_collision;
use combat::{update_blood_particles, update_damage_numbers, update_status_effects};
use console::*;
use game_state::{GamePlayEntity, GameState, GameStatePlugin};
use item::ItemPlugin;
use item::*;
use map::Map;
use map::editor::MapEditorPlugin;
use menu::MenuPlugin;
use rendering::update_billboards;
use scripting::{CVarRegistry, ScriptingPlugin};
use toolbar::Toolbar;
use ui::*;
use weapon::WeaponPlugin;
use world::WorldPlugin;

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
        .add_plugins(toolbar::ToolbarPlugin)
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
