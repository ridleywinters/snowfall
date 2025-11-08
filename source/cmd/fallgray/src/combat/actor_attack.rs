use crate::actor::{Actor, ActorAttackState};
use crate::camera::Player;
use bevy::prelude::*;

// Attack animation timing
const WINDUP_DURATION: f32 = 0.2;
const STRIKE_DURATION: f32 = 0.2;
const RECOVERY_DURATION: f32 = 0.2;
const ATTACK_TOTAL_DURATION: f32 = WINDUP_DURATION + STRIKE_DURATION + RECOVERY_DURATION;

// Damage is dealt at 50% through the attack (during strike phase)
const DAMAGE_TIMING: f32 = WINDUP_DURATION + STRIKE_DURATION * 0.5;

/// System to handle actor attacks on player
pub fn update_actor_attacks(
    mut actors: Query<(&mut Actor, &Transform)>,
    mut player_query: Query<(&mut Player, &Transform)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    combat_audio: Res<crate::combat::CombatAudio>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let Ok((mut player, player_transform)) = player_query.single_mut() else {
        return;
    };

    let player_pos = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y,
    );

    for (mut actor, actor_transform) in actors.iter_mut() {
        // Skip if actor is stunned
        if actor.stun_timer > 0.0 {
            continue;
        }

        let actor_pos = Vec2::new(actor_transform.translation.x, actor_transform.translation.y);
        let distance = actor_pos.distance(player_pos);

        // Progress attack timer
        actor.attack_timer += time.delta_secs();

        // Check if in attack state (managed by behavior)
        match actor.attack_state {
            ActorAttackState::Idle => {
                // Check if close enough to initiate attack and cooldown expired
                if distance <= actor.attack_range && actor.attack_timer >= actor.attack_cooldown {
                    actor.attack_state = ActorAttackState::WindingUp;
                    actor.attack_timer = 0.0;

                    // Play swing sound
                    play_actor_swing_sound(&mut commands, &combat_audio);
                }
            }

            ActorAttackState::WindingUp => {
                if actor.attack_timer >= WINDUP_DURATION {
                    actor.attack_state = ActorAttackState::Striking;
                }
            }

            ActorAttackState::Striking => {
                // Deal damage at precise timing
                if actor.attack_timer >= DAMAGE_TIMING
                    && actor.attack_timer < DAMAGE_TIMING + time.delta_secs()
                {
                    // Check if still in range
                    if distance <= actor.attack_range {
                        player.take_damage(actor.attack_damage as f32);

                        // Spawn visual/audio feedback
                        crate::combat::spawn_damage_number(
                            &mut commands,
                            &asset_server,
                            player_transform.translation,
                            actor.attack_damage,
                            false, // Not a crit
                        );

                        crate::combat::spawn_blood_particles(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            player_transform.translation,
                            5,
                        );

                        combat_audio.play_hit_sound(&mut commands, false);
                    }
                }

                if actor.attack_timer >= WINDUP_DURATION + STRIKE_DURATION {
                    actor.attack_state = ActorAttackState::Recovering;
                }
            }

            ActorAttackState::Recovering => {
                if actor.attack_timer >= ATTACK_TOTAL_DURATION {
                    actor.attack_state = ActorAttackState::Idle;
                    actor.attack_timer = 0.0; // Reset for cooldown
                }
            }
        }
    }
}

/// System to handle actor stun when damaged
pub fn update_actor_stun(mut actors: Query<&mut Actor>, time: Res<Time>) {
    for mut actor in actors.iter_mut() {
        if actor.stun_timer > 0.0 {
            actor.stun_timer -= time.delta_secs();
            if actor.stun_timer < 0.0 {
                actor.stun_timer = 0.0;
            }
        }
    }
}

/// System to apply stun when actor is hit (called from existing damage system)
/// This should be integrated with the existing update_weapon_swing_collision system
pub fn handle_actor_hit(actor: &mut Actor) {
    const STUN_DURATION: f32 = 0.3;
    actor.stun_timer = STUN_DURATION;

    // Reset attack state if in middle of attacking
    if actor.attack_state != ActorAttackState::Idle {
        actor.attack_state = ActorAttackState::Idle;
        actor.attack_timer = 0.0;
    }
}

/// System to animate actor attacks (scale and tint changes)
pub fn update_actor_attack_animation(
    mut actors: Query<(&Actor, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (actor, mut transform, material_handle) in actors.iter_mut() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            // Calculate animation progress within current state
            let progress = match actor.attack_state {
                ActorAttackState::WindingUp => {
                    (actor.attack_timer / WINDUP_DURATION).clamp(0.0, 1.0)
                }
                ActorAttackState::Striking => {
                    ((actor.attack_timer - WINDUP_DURATION) / STRIKE_DURATION).clamp(0.0, 1.0)
                }
                ActorAttackState::Recovering => {
                    ((actor.attack_timer - WINDUP_DURATION - STRIKE_DURATION) / RECOVERY_DURATION)
                        .clamp(0.0, 1.0)
                }
                ActorAttackState::Idle => 0.0,
            };

            match actor.attack_state {
                ActorAttackState::Idle => {
                    // Reset to normal
                    transform.scale = Vec3::splat(1.0);
                    material.base_color = Color::WHITE;
                }

                ActorAttackState::WindingUp => {
                    // Scale up 1.0 -> 1.1
                    let scale = 1.0 + progress * 0.1;
                    transform.scale = Vec3::splat(scale);
                    material.base_color = Color::WHITE;
                }

                ActorAttackState::Striking => {
                    // Stay at 1.1 scale, red tint
                    transform.scale = Vec3::splat(1.1);
                    material.base_color = Color::srgba(1.5, 0.8, 0.8, 1.0);
                }

                ActorAttackState::Recovering => {
                    // Scale back down 1.1 -> 1.0
                    let scale = 1.1 - progress * 0.1;
                    transform.scale = Vec3::splat(scale);
                    // Fade tint back to white
                    let tint_amount = 1.0 - progress;
                    material.base_color = Color::srgba(
                        1.0 + tint_amount * 0.5,
                        0.8 + tint_amount * 0.2,
                        0.8 + tint_amount * 0.2,
                        1.0,
                    );
                }
            }
        }
    }
}

/// Play random actor swing sound (reuses player sword sounds for now)
fn play_actor_swing_sound(commands: &mut Commands, combat_audio: &crate::combat::CombatAudio) {
    if let Some(sound) = &combat_audio.swing_sound {
        commands.spawn((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
    }
}
