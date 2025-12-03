use super::cvars::CVarRegistry;
use bevy::prelude::*;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app //
            .init_resource::<CVarRegistry>()
            .add_systems(
                PostStartup,
                (init_camera_cvars, save_cvars_on_startup).chain(),
            );
    }
}

/// Initialize camera-related CVars with default values
fn init_camera_cvars(mut cvars: ResMut<CVarRegistry>) {
    // Mouse sensitivity (radians per pixel of mouse movement)
    cvars.init_f32("mouse.sensitivity", 0.0005);

    // Arrow key sensitivity (radians per second)
    cvars.init_f32("arrow_sensitivity", 2.75);

    // Mouse smooth mode
    cvars.init_bool("mouse.smooth", true);

    // Mouse invert Y axis
    cvars.init_bool("mouse.invert_y", true);
}

fn save_cvars_on_startup(cvars: Res<CVarRegistry>) {
    if let Err(e) = cvars.save_to_yaml("data/cvars.yaml") {
        eprintln!("Failed to save cvars: {}", e);
    } else {
        println!("CVars saved to data/cvars.yaml");
    }
}
