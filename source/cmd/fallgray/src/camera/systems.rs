use super::mouse_look_settings::MouseLookSettings;
use super::player::{Health, Player};
use crate::collision::PLAYER_RADIUS;
use crate::console::ConsoleState;
use crate::game_state::GamePlayEntity;
use crate::map::Map;
use crate::scripting::CVarRegistry;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

pub fn update_camera_control_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    map: Res<Map>,
    console_state: Res<ConsoleState>,
    mouse_look: Res<MouseLookSettings>,
    cvars: Res<CVarRegistry>,
    mut query: Query<(&mut Transform, &mut Player)>,
    ui_interaction_query: Query<&Interaction>,
) {
    // Don't process camera controls if console is open
    if console_state.visible {
        return;
    }

    for (mut transform, mut player) in query.iter_mut() {
        let dt = time.delta_secs();

        // Mouse look input - only process if cursor is locked, console is closed, and not hovering UI
        let ui_hovered = ui_interaction_query.iter().any(|i| *i != Interaction::None);
        let can_mouse_look = mouse_look.cursor_locked && !console_state.visible && !ui_hovered;

        if can_mouse_look {
            // Read mouse sensitivity from CVar
            let mouse_sensitivity = cvars.get_f32("mouse.sensitivity");

            // Read invert_y setting from CVar
            let invert_y = cvars.get_bool("mouse.invert_y");
            let invert_factor = if invert_y { 1.0 } else { -1.0 };

            // Check if smooth mouse is enabled via CVar
            let smooth_enabled = cvars.get_bool("mouse.smooth");

            // Accumulate mouse motion
            for event in mouse_motion.read() {
                let yaw_input = -event.delta.x * mouse_sensitivity;
                let pitch_input = -event.delta.y * mouse_sensitivity * invert_factor;

                if smooth_enabled {
                    // Add to velocity accumulators for smooth mode
                    player.yaw_velocity += yaw_input;
                    player.pitch_velocity += pitch_input;
                } else {
                    // Direct mode - apply rotation immediately via arrow key delta variables
                    // (will be processed in the rotation section below)
                }
            }
        } else {
            // Clear mouse motion events when not using mouse look
            mouse_motion.clear();
        }

        // Check if modifier keys are pressed
        let ctrl_pressed =
            input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight);

        // Movement input (WASD + RF)
        // WASD moves in the XY plane, RF moves along Z axis
        let mut movement_xy = Vec2::ZERO; // Movement in XY plane
        let mut movement_z = 0.0; // Movement along Z axis

        if !ctrl_pressed {
            if input.pressed(KeyCode::KeyW) {
                movement_xy.y += 1.0;
            }
            if input.pressed(KeyCode::KeyS) {
                movement_xy.y -= 1.0;
            }
            if input.pressed(KeyCode::KeyA) {
                movement_xy.x -= 1.0;
            }
            if input.pressed(KeyCode::KeyD) {
                movement_xy.x += 1.0;
            }
            if input.pressed(KeyCode::KeyF) {
                movement_z -= 1.0;
            }
            if input.pressed(KeyCode::KeyR) {
                movement_z += 1.0;
            }
        }

        // Rotation input (Arrow keys)
        // Arrow left/right rotates around Z axis (yaw)
        // Arrow up/down changes pitch (looking up/down)
        // Read arrow sensitivity from CVar
        let arrow_sensitivity = cvars.get_f32("arrow_sensitivity");
        let mut yaw_delta = 0.0;
        let mut pitch_delta = 0.0;

        if input.pressed(KeyCode::ArrowLeft) {
            yaw_delta += arrow_sensitivity * dt;
        }
        if input.pressed(KeyCode::ArrowRight) {
            yaw_delta -= arrow_sensitivity * dt;
        }
        if input.pressed(KeyCode::ArrowUp) {
            pitch_delta += arrow_sensitivity * dt;
        }
        if input.pressed(KeyCode::ArrowDown) {
            pitch_delta -= arrow_sensitivity * dt;
        }

        // Get current yaw from the forward direction projected onto XY plane
        {
            let scale = if yaw_delta.abs() > 0.0 {
                0.25
            } else if movement_xy.length_squared() > 0.0 {
                0.1
            } else {
                0.0
            };

            let forward_3d = transform.forward().as_vec3();
            let forward_xy = Vec2::new(forward_3d.x, forward_3d.y);
            let yaw = forward_xy.y.atan2(forward_xy.x);

            let snap_increment = std::f32::consts::PI / 4.0;
            let mut yaw_snap = (yaw / snap_increment).round() * snap_increment;

            if yaw_delta < 0.0 && yaw_snap > yaw {
                yaw_snap -= snap_increment;
            } else if yaw_delta > 0.0 && yaw_snap < yaw {
                yaw_snap += snap_increment;
            }

            let max = scale * arrow_sensitivity * dt;
            yaw_delta += (yaw_snap - yaw).clamp(-max, max);
        }

        // Apply smooth mouse rotation (velocity-based)
        let smooth_enabled = cvars.get_bool("mouse.smooth");
        if smooth_enabled {
            let dt_factor = dt * 60.0; // Frame-rate independence (60 FPS baseline)

            // Clamp velocities to rotation limit
            player.yaw_velocity = player
                .yaw_velocity
                .clamp(-mouse_look.rotation_limit, mouse_look.rotation_limit);
            player.pitch_velocity = player
                .pitch_velocity
                .clamp(-mouse_look.rotation_limit, mouse_look.rotation_limit);

            // Add velocity to rotation deltas
            yaw_delta += player.yaw_velocity * dt_factor;
            pitch_delta += player.pitch_velocity * dt_factor;

            // Apply exponential decay to velocities
            let decay = mouse_look.smooth_factor.powf(dt_factor);
            player.yaw_velocity *= decay;
            player.pitch_velocity *= decay;
        }

        // Apply rotation
        if yaw_delta != 0.0 || pitch_delta != 0.0 {
            // Apply yaw rotation around the world Z axis
            if yaw_delta != 0.0 {
                let yaw_rotation = Quat::from_axis_angle(Vec3::Z, yaw_delta);
                transform.rotation = yaw_rotation * transform.rotation;
            }

            // Apply pitch rotation around the local X axis (right vector)
            if pitch_delta != 0.0 {
                // Calculate current pitch from the forward vector's Z component
                let forward_3d = transform.forward().as_vec3();
                let current_pitch = f32::asin(forward_3d.z.clamp(-1.0, 1.0));

                // Calculate new pitch and clamp to limits (from MouseLookSettings)
                let new_pitch = (current_pitch + pitch_delta)
                    .clamp(-mouse_look.pitch_limit, mouse_look.pitch_limit);
                let actual_pitch_delta = new_pitch - current_pitch;

                // Apply the pitch rotation around the local right (X) axis
                if actual_pitch_delta.abs() > 0.0001 {
                    let local_x = transform.right().as_vec3();
                    let pitch_rotation = Quat::from_axis_angle(local_x, actual_pitch_delta);
                    transform.rotation = pitch_rotation * transform.rotation;
                }
            }
        }

        // Apply XY plane movement in camera's local orientation (projected to XY plane)
        if movement_xy != Vec2::ZERO {
            movement_xy = movement_xy.normalize();

            // Get forward and right directions, but project them onto the XY plane
            let forward_3d = transform.forward();
            let right_3d = transform.right();

            // Project to XY plane by zeroing Z component and normalizing
            let forward_xy = Vec2::new(forward_3d.x, forward_3d.y).normalize_or_zero();
            let right_xy = Vec2::new(right_3d.x, right_3d.y).normalize_or_zero();

            let move_vec_xy = forward_xy * movement_xy.y + right_xy * movement_xy.x;

            // Calculate new position
            let new_x = transform.translation.x + move_vec_xy.x * player.speed * dt;
            let new_y = transform.translation.y + move_vec_xy.y * player.speed * dt;

            // Check collision before moving
            if map.can_move_to(new_x, new_y, PLAYER_RADIUS) {
                transform.translation.x = new_x;
                transform.translation.y = new_y;
            }
        }

        // Apply Z axis movement (no collision check for vertical movement)
        if movement_z != 0.0 {
            transform.translation.z += movement_z * player.speed * dt;
        }
    }
}

/// Spawn camera at given position and return its entity ID
pub fn spawn_camera(commands: &mut Commands, position: Vec3) -> Entity {
    commands
        .spawn((
            GamePlayEntity,
            Camera3d::default(),
            Camera {
                order: 1,
                ..default()
            },
            Transform::from_xyz(position.x, position.y, position.z).looking_at(
                Vec3::new(position.x - 1.0, position.y, position.z * 1.01),
                Vec3::Z,
            ),
            Player {
                speed: 32.0,
                yaw_velocity: 0.0,
                pitch_velocity: 0.0,
            },
            Health::new(100.0),
        ))
        .id()
}
