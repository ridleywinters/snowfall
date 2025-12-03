/// Mouse look settings resource
///
/// Controls cursor lock state, mouse smoothing, and rotation parameters
/// for first-person camera controls.
use bevy::prelude::*;

/// Resource for mouse look settings
#[derive(Resource)]
pub struct MouseLookSettings {
    /// Whether the cursor is currently locked for FPS controls
    ///
    /// NOTE: this is a bit confusing as this is a "should_cursor_be_locked" in some
    /// cases such as when the console is opened. When the console is opened, the
    /// cursor will be unlocked regardless of this setting -- but then this flag will
    /// be used to restore the cursor lock state when the console is closed.  There's
    /// probably a better way to handle this.
    pub cursor_locked: bool,

    /// Decay factor for smooth mouse (0.0-1.0, lower = more smoothing)
    pub smooth_factor: f32,

    /// Maximum rotation speed in radians per frame to prevent spinning
    pub rotation_limit: f32,

    /// Maximum pitch angle in radians (prevents looking too far up/down)
    pub pitch_limit: f32,
}

impl Default for MouseLookSettings {
    fn default() -> Self {
        Self {
            cursor_locked: false, // Start unlocked for safety
            smooth_factor: 0.5,
            rotation_limit: 0.35,
            pitch_limit: 70.0_f32.to_radians(), // ±70 degrees
        }
    }
}
