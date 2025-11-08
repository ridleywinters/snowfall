use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::ai::ActorBehavior;

/// Animation state for actor attacks
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActorAttackState {
    Idle,
    WindingUp,
    Striking,
    Recovering,
}

/// Component attached to actor entities in the game world
#[derive(Component)]
pub struct Actor {
    pub actor_type: String,
    pub health: f32,
    pub max_health: f32,
    pub scale: f32,
    /// Flat damage reduction
    pub armor: i32,
    /// Resistance to physical damage (0.0 = no resistance, 1.0 = immune)
    pub physical_resistance: f32,
    /// Collision radius for movement (3/4 of player radius)
    pub actor_radius: f32,
    /// Movement speed multiplier
    pub speed_multiplier: f32,
    /// AI behavior (if any)
    pub behavior: Option<Box<dyn ActorBehavior>>,
    /// Whether the actor is currently moving (for wiggle animation)
    pub is_moving: bool,
    /// Base Z position (for wiggle animation)
    pub base_z: f32,
    /// Attack damage dealt to player
    pub attack_damage: i32,
    /// Attack range in units
    pub attack_range: f32,
    /// Cooldown duration between attacks
    pub attack_cooldown: f32,
    /// Timer for tracking attack/cooldown progress
    pub attack_timer: f32,
    /// Timer for stun duration when hit
    pub stun_timer: f32,
    /// Current attack animation state
    pub attack_state: ActorAttackState,
}

/// Position data for actors in the map file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorPosition {
    pub x: f32,
    pub y: f32,
    pub actor_type: String,
}
