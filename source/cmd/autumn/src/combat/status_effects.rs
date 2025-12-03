/// Status effects system for combat
/// 
/// Handles temporary status effects on actors.

use bevy::prelude::*;
use super::damage::DamageType;

/// Status effect that can be applied to actors
#[derive(Component, Debug, Clone)]
pub struct StatusEffect {
    /// Type of effect (determines behavior)
    pub effect_type: StatusEffectType,
    
    /// Time remaining before effect expires
    pub duration: f32,
    
    /// Time between damage ticks (for DoT effects)
    pub tick_interval: f32,
    
    /// Time since last damage tick
    pub time_since_tick: f32,
    
    /// Damage per tick (for DoT effects)
    pub damage_per_tick: i32,
}

/// Types of status effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffectType {
    /// Frozen: slows movement (not implemented yet)
    Frozen,
}

impl StatusEffect {
    /// Create a new frozen effect
    pub fn frozen(duration: f32) -> Self {
        Self {
            effect_type: StatusEffectType::Frozen,
            duration,
            tick_interval: 0.0,
            time_since_tick: 0.0,
            damage_per_tick: 0,
        }
    }
    
    /// Check if this effect should deal damage this frame
    pub fn should_tick(&mut self, dt: f32) -> bool {
        self.time_since_tick += dt;
        if self.time_since_tick >= self.tick_interval && self.tick_interval > 0.0 {
            self.time_since_tick = 0.0;
            true
        } else {
            false
        }
    }
}

/// System to update status effects on actors
pub fn update_status_effects(
    time: Res<Time>,
    mut query: Query<(Entity, &mut StatusEffect, &mut crate::actor::Actor)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    
    for (entity, mut effect, mut actor) in query.iter_mut() {
        // Update duration
        effect.duration -= dt;
        
        // Apply damage if it's time to tick
        if effect.should_tick(dt) {
            actor.health -= effect.damage_per_tick as f32;
            
            // Print feedback (currently only Frozen, which doesn't deal damage)
            match effect.effect_type {
                StatusEffectType::Frozen => {
                    // Frozen doesn't deal damage
                }
            }
        }
        
        // Remove effect when expired
        if effect.duration <= 0.0 {
            commands.entity(entity).remove::<StatusEffect>();
            
            match effect.effect_type {
                StatusEffectType::Frozen => {
                    println!("{} thawed out", actor.actor_type);
                }
            }
        }
    }
}

/// Apply a status effect to an actor based on damage type
/// Returns true if an effect was applied
pub fn apply_status_effect(
    _commands: &mut Commands,
    _entity: Entity,
    damage_type: DamageType,
    _actor_type: &str,
) -> bool {
    match damage_type {
        DamageType::Physical => {
            // Physical damage doesn't apply status effects
            false
        }
    }
}
