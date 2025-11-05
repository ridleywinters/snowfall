use bevy::prelude::*;

/// Player/Camera entity marker with movement and rotation speeds
#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub rot_speed: f32,
}
