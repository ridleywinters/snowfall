use bevy::prelude::*;

/// Player/Camera entity marker with movement and rotation speeds
#[derive(Component)]
pub struct Player {
    pub speed: f32,

    /// Accumulators for smooth mouse movement
    pub yaw_velocity: f32,
    pub pitch_velocity: f32,

    /// Current health
    pub current_health: f32,
    /// Maximum health
    pub max_health: f32,
}

impl Player {
    pub fn new(speed: f32, max_health: f32) -> Self {
        Self {
            speed,
            yaw_velocity: 0.0,
            pitch_velocity: 0.0,
            current_health: max_health,
            max_health,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current_health > 0.0
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current_health = (self.current_health - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current_health = (self.current_health + amount).min(self.max_health);
    }
}
