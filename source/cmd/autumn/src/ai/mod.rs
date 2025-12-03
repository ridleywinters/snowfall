use crate::world::Map;
use bevy::prelude::*;

pub mod aggressive_behavior;
pub mod pathfinding;
#[cfg(test)]
mod pathfinding_test;
pub mod stand_behavior;
pub mod systems;
pub mod wander_behavior;

pub use systems::AIPlugin;

/// Minimal actor data needed by behaviors
pub struct ActorData {
    pub attack_state: crate::actor::ActorAttackState,
    pub attack_range: f32,
}

/// Trait for defining actor behaviors
pub trait ActorBehavior: Send + Sync {
    /// Update the behavior for the current frame
    /// Returns true if the actor is currently moving
    fn update(
        &mut self,
        transform: &mut Transform,
        map: &Map,
        delta_time: f32,
        speed_multiplier: f32,
        player_position: Option<Vec2>,
        actor: &ActorData,
    ) -> bool;

    /// Get the behavior label
    fn get_label(&self) -> &str;
}
