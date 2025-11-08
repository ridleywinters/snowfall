use bevy::prelude::*;
use crate::combat::AttackState;

/// Component for weapon sprite attached to camera
#[derive(Component)]
pub struct WeaponSprite {
    /// Current attack state (replaces is_swinging and collision_checked)
    pub attack_state: AttackState,

    /// Charge progress (0.0 to weapon's max_charge_time)
    pub charge_progress: f32,

    /// Entities already hit during this swing (prevents double-hits)
    pub hit_entities: std::collections::HashSet<Entity>,

    /// Currently equipped weapon type
    pub weapon_type: String,
}

impl Default for WeaponSprite {
    fn default() -> Self {
        Self {
            attack_state: AttackState::Idle,
            charge_progress: 0.0,
            hit_entities: std::collections::HashSet::new(),
            weapon_type: "sword".to_string(), // Default weapon
        }
    }
}
