mod camera_plugin;
mod cursor_toggle;
mod mouse_look_settings;
mod player;
mod player_light;
mod systems;

pub use camera_plugin::CameraPlugin;
pub use mouse_look_settings::MouseLookSettings;
pub use player::{Health, Player};
pub use player_light::{PlayerLightPlugin, spawn_player_lights};
pub use systems::{spawn_camera, update_camera_control_system};
