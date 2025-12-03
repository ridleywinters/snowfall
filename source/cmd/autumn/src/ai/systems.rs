use crate::actor::Actor;
use crate::combat::{update_actor_attack_animation, update_actor_attacks, update_actor_stun};
use crate::game_state::GameState;
use crate::world::Map;
use bevy::prelude::*;

const WIGGLE_AMPLITUDE: f32 = 0.1;
const WIGGLE_FREQUENCY: f32 = 10.0;

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_actor_stun,
                update_actor_behavior,
                add_actor_wiggle,
                update_actor_attacks,
                update_actor_attack_animation,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Update all actor behaviors
fn update_actor_behavior(
    mut actors: Query<(Entity, &mut Actor, &mut Transform), Without<crate::camera::Player>>,
    player_query: Query<&Transform, With<crate::camera::Player>>,
    map: Res<Map>,
    time: Res<Time>,
) {
    // Get player position if available
    let player_position = player_query
        .single()
        .ok()
        .map(|t| Vec2::new(t.translation.x, t.translation.y));

    for (_entity, mut actor, mut transform) in actors.iter_mut() {
        let speed = actor.speed_multiplier;
        // Extract necessary actor data before borrowing behavior mutably
        let actor_data = crate::ai::ActorData {
            attack_state: actor.attack_state,
            attack_range: actor.attack_range,
        };

        if let Some(ref mut behavior) = actor.behavior {
            let is_moving = behavior.update(
                &mut transform,
                &map,
                time.delta_secs(),
                speed,
                player_position,
                &actor_data,
            );
            actor.is_moving = is_moving;
        }
    }
}

/// Add wiggle animation to moving actors
fn add_actor_wiggle(mut actors: Query<(&Actor, &mut Transform)>, time: Res<Time>) {
    let elapsed = time.elapsed_secs();

    for (actor, mut transform) in actors.iter_mut() {
        if actor.is_moving {
            // Apply wiggle offset
            let wiggle_offset = (elapsed * WIGGLE_FREQUENCY).sin() * WIGGLE_AMPLITUDE;
            // Store base position and apply wiggle
            // For simplicity, wiggle on a perpendicular axis
            transform.translation.z = actor.base_z + wiggle_offset;
        } else {
            // Reset to base position
            transform.translation.z = actor.base_z;
        }
    }
}
