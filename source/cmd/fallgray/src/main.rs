mod actor;
mod ai;
mod camera;
mod collision;
mod combat;
mod console;
mod game_state;
mod game_state_systems;
mod item;
mod logging;
mod map;
#[cfg(test)]
mod map_test;
mod menu_ui;
mod scripting;
mod texture_loader;
mod toolbar;
mod ui;
mod ui_styles;
use actor::*;
use ai::AIPlugin;
use bevy::prelude::*;
use camera::{CameraPlugin, Player, PlayerLightPlugin, spawn_camera, spawn_player_lights};
use collision::check_circle_collision;
use combat::{
    AttackState, CombatAudio, CombatInput, StateTransition, WeaponDefinitions, apply_status_effect,
    play_hit_sound, play_swing_sound, spawn_blood_particles, spawn_damage_number,
    update_blood_particles, update_camera_shake, update_damage_numbers, update_status_effects,
};
use console::*;
use game_state::GamePlayEntity;
use game_state::GameState;
use game_state_systems::*;
use item::*;
use logging::ActorLoggingSystem;
use map::Map;
use menu_ui::*;
use scripting::{CVarRegistry, ScriptingPlugin};
use std::f32::consts::FRAC_PI_2;
use texture_loader::{load_image_texture, load_weapon_texture};
use toolbar::Toolbar;
use ui::*;

// MapFile and MapData are now defined in map.rs

fn main() {
    // Get asset path from REPO_ROOT environment variable
    let asset_path = std::env::var("REPO_ROOT")
        .map(|repo_root| format!("{}/source/assets", repo_root))
        .unwrap();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::asset::AssetPlugin {
                    file_path: asset_path,
                    ..default()
                })
                .set(bevy::window::WindowPlugin {
                    primary_window: Some(bevy::window::Window {
                        title: "Fallgray".into(),
                        resolution: (1920, 1080).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .add_systems(Startup, (log_startup, setup_ui_camera))
        .add_plugins(ScriptingPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PlayerLightPlugin)
        .add_plugins(AIPlugin)
        .add_plugins(ConsolePlugin {})
        .add_plugins(toolbar::ToolbarPlugin)
        // Main Menu systems
        .add_systems(
            OnEnter(GameState::MainMenu),
            (spawn_main_menu, unlock_cursor_on_menu),
        )
        .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
        // Playing state systems
        .add_systems(OnEnter(GameState::Playing), (startup_system, startup_ui))
        .add_systems(
            Update,
            (
                update_weapon_swing,
                update_weapon_swing_collision,
                update_actor_death,
                update_actor_health_indicators,
                update_camera_shake.after(camera::update_camera_control_system),
                update_damage_numbers,
                update_blood_particles,
                update_status_effects,
                update_ui,
                update_billboards,
                update_spawn_item_on_click,
                update_save_map_on_input,
                update_check_item_collision,
                detect_player_death,
                initialize_actor_logs,
                periodic_flush_actor_logs,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            OnExit(GameState::Playing),
            (cleanup_actor_logging, cleanup_game_entities),
        )
        // Game Over systems
        .add_systems(
            OnEnter(GameState::GameOver),
            (spawn_game_over, unlock_cursor_on_menu),
        )
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over)
        // Menu button systems (active in all menu states)
        .add_systems(
            Update,
            (handle_menu_buttons, update_button_visuals).run_if(not(in_state(GameState::Playing))),
        )
        .run();
}

fn log_startup(current_state: Res<State<GameState>>) {
    info!("=== Game Starting ===");
    info!("Initial game state: {:?}", current_state.get());
}

/// Setup a persistent UI camera for menus
fn setup_ui_camera(mut commands: Commands) {
    info!("Spawning persistent UI camera for menus");
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
    ));
}

#[derive(Component)]
struct Billboard;

#[derive(Component)]
struct GroundPlane;

// Weapon swing components
#[derive(Component)]
struct WeaponSprite {
    /// Current attack state (replaces is_swinging and collision_checked)
    attack_state: AttackState,

    /// Charge progress (0.0 to weapon's max_charge_time)
    charge_progress: f32,

    /// Entities already hit during this swing (prevents double-hits)
    hit_entities: std::collections::HashSet<Entity>,

    /// Currently equipped weapon type
    weapon_type: String,
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

// Easing functions for weapon swing
fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

fn startup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut cvars: ResMut<CVarRegistry>,
) {
    // Create a 512x512 plane in the XY plane at z=0
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(512.0, 512.0));
    let plane_material2 = materials.add(StandardMaterial {
        base_color_texture: Some(load_image_texture(
            &asset_server,
            "base/textures/stone_1.png",
        )),
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        metallic: 0.0,
        reflectance: 0.0,
        uv_transform: bevy::math::Affine2::from_scale(Vec2::new(64.0, 64.0)),
        ..default()
    });

    commands.spawn((
        GamePlayEntity,
        Mesh3d(plane_mesh.clone()),
        MeshMaterial3d(plane_material2.clone()),
        // Rotate 90 degrees around X to make it XY plane (facing Z)
        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2))
            .with_translation(Vec3::new(256.0, 256.0, 0.0)),
        GroundPlane,
    ));

    commands.spawn((
        GamePlayEntity,
        Mesh3d(plane_mesh.clone()),
        MeshMaterial3d(plane_material2.clone()),
        Transform::from_rotation(Quat::from_rotation_x(3.0 * FRAC_PI_2))
            .with_translation(Vec3::new(256.0, 256.0, 16.0)),
    ));

    // Load item definitions
    let filename = std::env::var("REPO_ROOT")
        .map(|repo_root| format!("{}/source/assets/base/items/items.yaml", repo_root))
        .unwrap_or_else(|_| "data/item_definitions.yaml".to_string());
    let item_defs_yaml = std::fs::read_to_string(&filename)
        .unwrap_or_else(|_| panic!("Failed to read {}", filename));
    let item_defs_file: ItemDefinitionsFile = serde_yaml::from_str(&item_defs_yaml)
        .unwrap_or_else(|_| panic!("Failed to parse {}", filename));
    let item_definitions = ItemDefinitions {
        items: item_defs_file.items,
    };

    // Load actor definitions
    let actor_filename = "data/actor_definitions.yaml".to_string();
    let actor_defs_yaml = std::fs::read_to_string(&actor_filename)
        .unwrap_or_else(|_| panic!("Failed to read {}", actor_filename));
    let actor_defs_file: ActorDefinitionsFile = serde_yaml::from_str(&actor_defs_yaml)
        .unwrap_or_else(|_| panic!("Failed to parse {}", actor_filename));
    let actor_definitions = ActorDefinitions {
        actors: actor_defs_file.actors,
    };

    // Load weapon definitions
    let weapon_filename = std::env::var("REPO_ROOT")
        .map(|repo_root| format!("{}/source/cmd/fallgray/data/weapons.yaml", repo_root))
        .unwrap_or_else(|_| "data/weapons.yaml".to_string());
    let weapon_definitions = WeaponDefinitions::load_from_file(&weapon_filename)
        .unwrap_or_else(|e| panic!("Failed to load weapons: {}", e));

    // Register weapon CVars for runtime tuning
    weapon_definitions.register_cvars(&mut cvars);

    // Load map from file and spawn all entities
    let map = Map::load_from_file(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        &item_definitions,
        &actor_definitions,
    )
    .expect("Failed to load map");

    // Initialize actor logging system
    match ActorLoggingSystem::create_session() {
        Ok(logging_system) => {
            info!(
                "Actor logging initialized: {:?}",
                logging_system.session_folder
            );
            commands.insert_resource(logging_system);
        }
        Err(e) => {
            eprintln!("Failed to initialize actor logging: {}", e);
        }
    }

    commands.insert_resource(map);
    commands.insert_resource(item_definitions);
    commands.insert_resource(actor_definitions);

    commands.insert_resource(bevy::light::AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
        affects_lightmapped_meshes: false,
    });

    let player_start_pos = Vec3::new(256.0 + 4.0, 200.0 + 4.0, 4.8);

    // Spawn camera and player lights
    let camera_entity = spawn_camera(&mut commands, player_start_pos);
    spawn_player_lights(&mut commands, player_start_pos);

    // Spawn weapon sprite as child of camera for first-person view
    // (Do this before inserting weapon_definitions as resource)
    spawn_weapon_sprite(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        camera_entity,
        &weapon_definitions,
        &cvars,
    );

    // Now insert weapon_definitions as resource
    commands.insert_resource(weapon_definitions);

    // Load combat audio
    let combat_audio = CombatAudio::load_sounds(&asset_server);
    commands.insert_resource(combat_audio);
}

fn update_billboards(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut billboard_query: Query<&mut Transform, (With<Billboard>, Without<Camera3d>)>,
) {
    if let Ok(camera_transform) = camera_query.single() {
        let camera_pos = camera_transform.translation;

        for mut billboard_transform in billboard_query.iter_mut() {
            let billboard_pos = billboard_transform.translation;

            // Calculate direction from billboard to camera (in XY plane)
            let to_camera = Vec2::new(
                camera_pos.x - billboard_pos.x,
                camera_pos.y - billboard_pos.y,
            );

            // The plane's normal starts pointing in X direction (Dir3::X)
            // Calculate angle around Z axis to rotate the normal to face the camera
            let angle = to_camera.y.atan2(to_camera.x);

            // Rotate around Z axis so the plane normal points toward camera
            billboard_transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn update_weapon_swing(
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
                    play_swing_sound(&mut commands, &combat_audio);
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

fn update_weapon_swing_collision(
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
                combat::calculate_damage(&weapon_def, charge_ratio, actor.armor, resistance);

            // Apply damage
            actor.health -= damage_result.amount as f32;

            // Apply stun when hit
            combat::handle_actor_hit(&mut actor);

            // Spawn visual feedback
            // Camera shake
            if damage_result.critical {
                commands
                    .entity(camera_entity)
                    .insert(combat::CameraShake::critical_shake());
            } else {
                commands
                    .entity(camera_entity)
                    .insert(combat::CameraShake::hit_shake());
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
            play_hit_sound(&mut commands, &combat_audio, damage_result.critical);

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

fn spawn_weapon_sprite(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    camera_entity: Entity,
    weapon_definitions: &WeaponDefinitions,
    cvars: &CVarRegistry,
) {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};

    let sprite_material = materials.add(StandardMaterial {
        base_color_texture: Some(load_weapon_texture(asset_server, "base/icons/sword.png")),
        base_color: Color::WHITE,
        alpha_mode: bevy::render::alpha::AlphaMode::Blend,
        unlit: true, // Keep weapon bright and visible
        cull_mode: None,
        ..default()
    });

    let scale = 0.5; // Smaller scale for weapon icon

    let mut weapon_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Create a quad for the weapon sprite
    let positions = vec![
        [-scale, -scale, 0.0], // bottom-left
        [scale, -scale, 0.0],  // bottom-right
        [scale, scale, 0.0],   // top-right
        [-scale, scale, 0.0],  // top-left
    ];
    weapon_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    weapon_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 4]);

    let uvs = vec![
        [0.0, 1.0], // bottom-left
        [1.0, 1.0], // bottom-right
        [1.0, 0.0], // top-right
        [0.0, 0.0], // top-left
    ];
    weapon_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    weapon_mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));

    // Get default weapon (sword) rest position from weapon definition
    let default_weapon_type = "sword";
    let weapon_def = weapon_definitions
        .get_with_cvars(default_weapon_type, cvars)
        .expect("Failed to load default weapon definition");

    let rest_pos = weapon_def.rest_keyframe.position;

    let weapon_entity = commands
        .spawn((
            GamePlayEntity,
            Mesh3d(meshes.add(weapon_mesh)),
            MeshMaterial3d(sprite_material),
            Transform::from_translation(rest_pos),
            WeaponSprite::default(),
        ))
        .id();

    // Parent weapon to camera
    commands
        .entity(camera_entity)
        .add_children(&[weapon_entity]);
}

fn update_spawn_item_on_click(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&bevy::window::Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    ground_query: Query<&GlobalTransform, With<GroundPlane>>,
    ui_interaction_query: Query<&Interaction>,
    mut map: ResMut<Map>,
    toolbar: Res<Toolbar>,
    item_definitions: Res<ItemDefinitions>,
    console_state: Res<ConsoleState>,
) {
    // Don't spawn items if console is open
    if console_state.visible {
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Left) && !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    // Only spawn items if slot 2 or 3 is active
    if toolbar.active_slot != 2 && toolbar.active_slot != 3 {
        return;
    }

    // Check if any UI element is being interacted with
    for interaction in ui_interaction_query.iter() {
        if *interaction != Interaction::None {
            return;
        }
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(_ground_transform) = ground_query.single() else {
        return;
    };

    // Convert cursor position to a ray in world space
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // The ground plane is at z=0 in world space (XY plane)
    // Ray equation: point = origin + t * direction
    // For intersection with z=0 plane: origin.z + t * direction.z = 0
    // Therefore: t = -origin.z / direction.z

    if ray.direction.z.abs() < 0.001 {
        // Ray is parallel to ground plane
        return;
    }

    let t = -ray.origin.z / ray.direction.z;

    if t < 0.0 {
        // Intersection is behind the camera
        return;
    }

    let intersection = ray.origin + ray.direction * t;

    // Force to a grid of 2x2 units
    let grid_x = (intersection.x / 2.0).floor() as i32;
    let grid_y = (intersection.y / 2.0).floor() as i32;

    // Check if there's already an item at this position
    if map.get_item_at(grid_x, grid_y).is_some() {
        return;
    }

    // Calculate world position
    let world_x = grid_x as f32 * 2.0 + 1.0;
    let world_y = grid_y as f32 * 2.0 + 1.0;

    // Select item based on active toolbar slot
    let item_key = match toolbar.active_slot {
        2 => "apple",
        3 => "coin-gold",
        _ => "apple", // Fallback (shouldn't happen due to earlier check)
    };

    // Spawn item using map
    map.spawn_item(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        &item_definitions,
        world_x,
        world_y,
        item_key,
    );
}

fn update_save_map_on_input(
    input: Res<ButtonInput<KeyCode>>,
    map: Res<Map>,
    console_state: Res<ConsoleState>,
) {
    // Don't save map if console is open
    if console_state.visible {
        return;
    }

    // Press Ctrl+S to save the map
    if (input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight))
        && input.just_pressed(KeyCode::KeyS)
    {
        if let Err(e) = map.save_to_yaml() {
            eprintln!("Failed to save map: {}", e);
        } else {
            println!(
                "Map saved successfully with {} items!",
                map.item_world_positions.len()
            );
        }
    }
}

fn update_actor_death(
    mut commands: Commands,
    actor_query: Query<(Entity, &Actor)>,
    mut stats: ResMut<PlayerStats>,
    mut cvars: ResMut<CVarRegistry>,
    mut map: ResMut<Map>,
    actor_definitions: Res<ActorDefinitions>,
) {
    for (entity, actor) in actor_query.iter() {
        if actor.health <= 0.0 {
            // Get the actor definition to run the on_death script
            if let Some(actor_def) = actor_definitions.actors.get(&actor.actor_type) {
                if !actor_def.on_death.is_empty() {
                    let output =
                        scripting::process_script(&actor_def.on_death, &mut stats, &mut cvars);
                    for line in &output {
                        println!("{}", line);
                    }
                }
            }

            println!("{} defeated!", actor.actor_type);

            // Unregister from map
            map.unregister_actor(entity);

            // Despawn actor (children like health indicator will be handled by bevy)
            commands.entity(entity).despawn();
        }
    }
}

fn update_actor_health_indicators(
    mut materials: ResMut<Assets<StandardMaterial>>,
    actor_query: Query<(&MeshMaterial3d<StandardMaterial>, &Actor)>,
) {
    for (material_handle, actor) in actor_query.iter() {
        let health_percentage = actor.health / actor.max_health;
        let should_show_indicator = health_percentage < 0.4;

        if let Some(material) = materials.get_mut(&material_handle.0) {
            if should_show_indicator {
                // Tint sprite 50% red
                material.base_color = Color::srgb(1.0, 0.5, 0.5);
            } else {
                // Normal white color
                material.base_color = Color::WHITE;
            }
        }
    }
}

fn update_check_item_collision(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    item_query: Query<(Entity, &Transform, &Item)>,
    mut stats: ResMut<PlayerStats>,
    mut cvars: ResMut<CVarRegistry>,
    mut map: ResMut<Map>,
    item_definitions: Res<ItemDefinitions>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (entity, item_transform, item) in item_query.iter() {
        let item_pos = item_transform.translation;

        if check_circle_collision(player_pos, item_pos, item.interaction_radius) {
            // Find the item type from the map
            let item_type = map
                .item_world_positions
                .iter()
                .find(|(pos, _)| {
                    (pos.x - item_pos.x).abs() < 0.1 && (pos.y - item_pos.y).abs() < 0.1
                })
                .map(|(_, item_type)| item_type.as_str())
                .unwrap_or("apple");

            // Get the item definition and process the script
            if let Some(item_def) = item_definitions.items.get(item_type) {
                println!("Item script: {}", item_def.script);
                let output = scripting::process_script(&item_def.script, &mut stats, &mut cvars);
                for line in &output {
                    println!("{}", line);
                }
            }

            // Remove item from world
            commands.entity(entity).despawn();

            // Remove from map tracking
            let grid_x = (item_pos.x / 2.0).floor() as i32;
            let grid_y = (item_pos.y / 2.0).floor() as i32;
            map.unregister_item(grid_x, grid_y);

            println!("Collected item! Fatigue: {}", stats.stamina);
        }
    }
}
