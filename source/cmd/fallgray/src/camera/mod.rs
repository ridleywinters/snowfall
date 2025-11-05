mod camera_plugin;
mod components;
mod player;
mod systems;

pub use camera_plugin::CameraPlugin;
pub use components::*;
pub use player::Player;
pub use systems::{spawn_camera, spawn_player_lights};
