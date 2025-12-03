use bevy::prelude::*;
use crate::camera::CameraShake;
use crate::combat::{
    AttackState, CombatAudio, CombatInput, StateTransition, WeaponDefinitions,
    apply_status_effect, spawn_blood_particles, spawn_damage_number,
};
use crate::console::ConsoleState;
use crate::item::Item;
use crate::rendering::Billboard;
use crate::scripting::CVarRegistry;
use crate::hud::Toolbar;
use crate::actor::Actor;
use super::components::WeaponSprite;
use super::easing::{ease_in_out_cubic, ease_out_quad};

/// System to update weapon swing animation and state
pub fn update_weapon_swing(
    mut commands: Commands,
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    toolbar: Res<Toolbar>,
    console_state: Res<ConsoleState>,
    cvars: Res<CVarRegistry>,
    weapon_definitions: Res<WeaponDefinitions>,
    combat_audio: Res<CombatAudio>,
    mut weapon_query: Query<(&mut Transform, &mut WeaponSprite, &mut Visibility)>,
    ui_interaction_query: Query<&Interaction>,
) {
    for (mut transform, mut weapon, mut visibility) in weapon_query.iter_mut() {
        // Only show the weapon sprite when slot 1 is active
        *visibility = if toolbar.active_slot == 1 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        // Get weapon definition with current CVar values
        let Some(weapon_def) = weapon_definitions.get_with_cvars(&weapon.weapon_type, &cvars)
        else {
            continue;
        };

        // Build combat input state
        let input = CombatInput {
            attack_pressed: (mouse_button.just_pressed(MouseButton::Left)
                || keyboard.just_pressed(KeyCode::Space))
                && toolbar.active_slot == 1
                && !console_state.visible
                && !ui_interaction_query.iter().any(|i| *i != Interaction::None),
            attack_held: (mouse_button.pressed(MouseButton::Left)
                || keyboard.pressed(KeyCode::Space))
                && toolbar.active_slot == 1
                && !console_state.visible,
        };

        // Handle charging when idle
        if matches!(weapon.attack_state, AttackState::Idle) {
            if input.attack_held {
                weapon.charge_progress += time.delta_secs();
                weapon.charge_progress = weapon.charge_progress.min(weapon_def.max_charge_time);

                // Add charge vibration to weapon position
                let charge_ratio = weapon.charge_progress / weapon_def.max_charge_time;
                let shake_offset = Vec3::new(
                    0.0,
                    0.0,
                    (time.elapsed_secs() * 10.0).sin() * charge_ratio * 0.05,
                );
                transform.translation += shake_offset;
            }
        }

        // Update attack state
        let transition = weapon
            .attack_state
            .update(time.delta_secs(), &input, &weapon_def);

        match transition {
            StateTransition::To(new_state) => {
                // Play swing sound when starting a new attack
                if matches!(new_state, AttackState::Windup { .. }) {
                    combat_audio.play_swing_sound(&mut commands);
                }

                // Clear hit list when returning to idle
                if matches!(new_state, AttackState::Idle) {
                    weapon.hit_entities.clear();
                    weapon.charge_progress = 0.0;
                }
                weapon.attack_state = new_state;
            }
            StateTransition::TriggerHitDetection => {
                // Hit detection will be handled by collision system
            }
            StateTransition::Stay => {}
        }

        // Animate weapon based on current state
        let overall_progress = weapon.attack_state.get_overall_progress();

        // Get keyframe positions from weapon definition
        let (current_pos, current_rot) = if overall_progress < 0.15 {
            // Windup phase
            let t = ease_out_quad(overall_progress / 0.15);
            let pos = weapon_def
                .rest_keyframe
                .position
                .lerp(weapon_def.windup_keyframe.position, t);
            let rot_z = weapon_def.rest_keyframe.rotation.0
                + (weapon_def.windup_keyframe.rotation.0 - weapon_def.rest_keyframe.rotation.0) * t;
            let rot_y = weapon_def.rest_keyframe.rotation.1
                + (weapon_def.windup_keyframe.rotation.1 - weapon_def.rest_keyframe.rotation.1) * t;
            (pos, (rot_z, rot_y))
        } else if overall_progress < 0.50 {
            // Swing phase
            let t = ease_in_out_cubic((overall_progress - 0.15) / 0.35);
            let pos = weapon_def
                .windup_keyframe
                .position
                .lerp(weapon_def.swing_keyframe.position, t);
            let rot_z = weapon_def.windup_keyframe.rotation.0
                + (weapon_def.swing_keyframe.rotation.0 - weapon_def.windup_keyframe.rotation.0)
                    * t;
            let rot_y = weapon_def.windup_keyframe.rotation.1
                + (weapon_def.swing_keyframe.rotation.1 - weapon_def.windup_keyframe.rotation.1)
                    * t;
            (pos, (rot_z, rot_y))
        } else if overall_progress < 0.80 {
            // Thrust phase
            let t = ease_in_out_cubic((overall_progress - 0.50) / 0.30);
            let pos = weapon_def
                .swing_keyframe
                .position
                .lerp(weapon_def.thrust_keyframe.position, t);
            let rot_z = weapon_def.swing_keyframe.rotation.0
                + (weapon_def.thrust_keyframe.rotation.0 - weapon_def.swing_keyframe.rotation.0)
                    * t;
            let rot_y = weapon_def.swing_keyframe.rotation.1
                + (weapon_def.thrust_keyframe.rotation.1 - weapon_def.swing_keyframe.rotation.1)
                    * t;
            (pos, (rot_z, rot_y))
        } else {
            // Recovery phase
            let t = ease_out_quad((overall_progress - 0.80) / 0.20);
            let pos = weapon_def
                .thrust_keyframe
                .position
                .lerp(weapon_def.rest_keyframe.position, t);
            let rot_z = weapon_def.thrust_keyframe.rotation.0
                + (weapon_def.rest_keyframe.rotation.0 - weapon_def.thrust_keyframe.rotation.0) * t;
            let rot_y = weapon_def.thrust_keyframe.rotation.1
                + (weapon_def.rest_keyframe.rotation.1 - weapon_def.thrust_keyframe.rotation.1) * t;
            (pos, (rot_z, rot_y))
        };

        // Apply animation to transform
        transform.translation = current_pos;
        transform.rotation =
            Quat::from_rotation_z(current_rot.0) * Quat::from_rotation_y(current_rot.1);
    }
}

/// System to check weapon collision with actors
pub fn update_weapon_swing_collision(
    mut commands: Commands,
    camera_query: Query<(Entity, &Transform), With<Camera3d>>,
    mut actor_query: Query<(Entity, &Transform, &mut Actor), (With<Billboard>, Without<Item>)>,
    mut weapon_query: Query<&mut WeaponSprite>,
    weapon_definitions: Res<WeaponDefinitions>,
    cvars: Res<CVarRegistry>,
    asset_server: Res<AssetServer>,
    combat_audio: Res<CombatAudio>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((camera_entity, camera_transform)) = camera_query.single() else {
        return;
    };

    for mut weapon in weapon_query.iter_mut() {
        // Only check collision when the attack state indicates hit detection is active
        if !weapon.attack_state.is_hit_active() {
            continue;
        }

        // Get weapon definition with current CVar values
        let Some(weapon_def) = weapon_definitions.get_with_cvars(&weapon.weapon_type, &cvars)
        else {
            continue;
        };

        // Get camera position and forward direction
        let camera_pos = camera_transform.translation;
        let forward = camera_transform.forward().as_vec3();

        // Project forward direction to XY plane and normalize
        let forward_xy = Vec2::new(forward.x, forward.y).normalize_or_zero();

        // Use weapon-specific hitbox dimensions
        let check_distance = weapon_def.range;
        let check_width = weapon_def.hitbox_width / 2.0;
        let check_height = weapon_def.hitbox_height;

        // Calculate right vector perpendicular to forward (for width check)
        let right_xy = Vec2::new(-forward_xy.y, forward_xy.x);

        // Check all actors (excluding items)
        for (entity, actor_transform, mut actor) in actor_query.iter_mut() {
            // Skip if already hit during this attack
            if weapon.hit_entities.contains(&entity) {
                continue;
            }

            let actor_pos = actor_transform.translation;
            let actor_xy = Vec2::new(actor_pos.x, actor_pos.y);

            // Vector from camera to actor
            let to_actor = actor_xy - Vec2::new(camera_pos.x, camera_pos.y);

            // Project onto forward direction to get distance along view direction
            let forward_distance = to_actor.dot(forward_xy);

            // Only check actors in front of player
            if forward_distance < 0.0 {
                continue;
            }

            // Check if actor is within the collision box
            // Distance check: is it within reach?
            if forward_distance > check_distance + actor.scale {
                continue;
            }

            // Width check: project onto right vector to get lateral distance
            let lateral_distance = to_actor.dot(right_xy).abs();

            if lateral_distance > check_width + actor.scale {
                continue;
            }

            // Z-axis height check (world uses Z+ as up)
            let height_difference = (actor_pos.z - camera_pos.z).abs();
            if height_difference > check_height {
                continue;
            }

            // Actor is within hitbox - calculate and apply damage
            weapon.hit_entities.insert(entity);

            // Calculate charge ratio (normalized by weapon's max charge time)
            let charge_ratio = (weapon.charge_progress / weapon_def.max_charge_time).min(1.0);

            // Get target resistance based on damage type
            let resistance = actor.physical_resistance;

            // Calculate damage
            let damage_result =
                crate::combat::calculate_damage(&weapon_def, charge_ratio, actor.armor, resistance);

            // Apply damage
            actor.health -= damage_result.amount as f32;

            // Apply stun when hit
            crate::combat::handle_actor_hit(&mut actor);

            // Spawn visual feedback
            // Camera shake
            if damage_result.critical {
                commands
                    .entity(camera_entity)
                    .insert(CameraShake::critical_shake());
            } else {
                commands
                    .entity(camera_entity)
                    .insert(CameraShake::hit_shake());
            }

            // Blood particles
            spawn_blood_particles(
                &mut commands,
                &mut meshes,
                &mut materials,
                actor_pos,
                if damage_result.critical { 10 } else { 5 },
            );

            // Damage number
            spawn_damage_number(
                &mut commands,
                &asset_server,
                actor_pos,
                damage_result.amount,
                damage_result.critical,
            );

            // Play hit sound
            combat_audio.play_hit_sound(&mut commands, damage_result.critical);

            // Apply status effect based on damage type
            apply_status_effect(
                &mut commands,
                entity,
                damage_result.damage_type,
                &actor.actor_type,
            );

            // Print hit feedback
            if damage_result.critical {
                println!(
                    "CRITICAL HIT! {} damage to {} (health: {:.0}/{:.0})",
                    damage_result.amount, actor.actor_type, actor.health, actor.max_health
                );
            } else {
                println!(
                    "Hit {} for {} damage (health: {:.0}/{:.0})",
                    actor.actor_type, damage_result.amount, actor.health, actor.max_health
                );
            }
        }
    }
}
