use bevy::prelude::*;

/// Light that follows the player with a fixed offset
#[derive(Component)]
pub struct PlayerLight {
    pub offset: Vec3,
}

/// Animation state for torch light color flickering
#[derive(Component)]
pub struct LightColorAnimation {
    pub time: f32,
    pub speed: f32,
}

impl Default for LightColorAnimation {
    fn default() -> Self {
        Self {
            time: 0.0,
            speed: 1.0,
        }
    }
}

impl LightColorAnimation {
    pub fn new(time: f32, speed: f32) -> Self {
        Self { time, speed }
    }
}
