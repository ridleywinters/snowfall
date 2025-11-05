mod actor;
mod camera;
mod collision;
mod console;
mod item;
mod scripting;
mod texture_loader;
mod toolbar;
mod ui;
mod ui_styles;

use actor::*;
use bevy::prelude::*;
use camera::{CameraPlugin, Player, spawn_camera, spawn_player_lights};
use collision::{CollisionMap, check_circle_collision};
use console::*;
use item::*;
use scripting::{CVarRegistry, ScriptingPlugin};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use texture_loader::{load_image_texture, load_weapon_texture};
use toolbar::Toolbar;
use ui::*;

#[derive(Deserialize, Serialize)]
struct MapFile {
    map: MapData,
}

#[derive(Deserialize, Serialize)]
struct MapData {
    grid: Vec<String>,
    #[serde(default)]
    items: Vec<ItemPosition>,
    #[serde(default)]
    actors: Vec<ActorPosition>,
}

fn main() {
    // Get asset path from REPO_ROOT environment variable
    let asset_path = std::env::var("REPO_ROOT")
        .map(|repo_root| format!("{}/source/assets", repo_root))
        .unwrap_or_else(|_| "assets".to_string());

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
        .add_plugins(ScriptingPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(ConsolePlugin {})
        .add_plugins(toolbar::ToolbarPlugin)
        .add_systems(
            Startup,
            (
                startup_system, //
                startup_ui,
            ),
        )
        .add_systems(
            Update,
            (
                update_weapon_swing,
                update_weapon_swing_collision,
                update_actor_death,
                update_actor_health_indicators,
                update_ui,
                update_billboards,
                update_spawn_item_on_click,
                update_save_map_on_input,
                update_check_item_collision,
            ),
        )
        .run();
}

#[derive(Component)]
struct Billboard;

#[derive(Component)]
struct GroundPlane;

// Weapon swing components
#[derive(Component)]
struct WeaponSprite {
    swing_timer: Timer,
    is_swinging: bool,
    collision_checked: bool, // Track if we've checked collision this swing
}

impl Default for WeaponSprite {
    fn default() -> Self {
        Self {
            swing_timer: Timer::from_seconds(0.4, TimerMode::Once),
            is_swinging: false,
            collision_checked: false,
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

/// Initialize console variables used by the weapon system
///
/// This is done to allow the weapon animation parameters to be
/// at runtime for immediate testing.  
fn initialize_weapon_cvars(cvars: &mut CVarRegistry) {
    // Weapon animation cvars
    cvars.init_f32("weapon.swing_duration", 0.4);
    cvars.init_f32("weapon.windup_end", 0.15);
    cvars.init_f32("weapon.swing_end", 0.80);
    cvars.init_f32("weapon.followthrough_end", 1.0);
    cvars.init_f32("weapon.rest_pos_x", 0.9);
    cvars.init_f32("weapon.rest_pos_y", -0.45);
    cvars.init_f32("weapon.rest_pos_z", -1.2);
    cvars.init_f32("weapon.rest_rotation_z", 0.0);
    cvars.init_f32("weapon.rest_rotation_y", 0.0);
    cvars.init_f32("weapon.windup_pos_x", 0.7);
    cvars.init_f32("weapon.windup_pos_y", -0.35);
    cvars.init_f32("weapon.windup_pos_z", -0.8);
    cvars.init_f32("weapon.windup_rotation_z", -0.5);
    cvars.init_f32("weapon.windup_rotation_y", 0.8);
    cvars.init_f32("weapon.thrust_pos_x", 0.3);
    cvars.init_f32("weapon.thrust_pos_y", -0.45);
    cvars.init_f32("weapon.thrust_pos_z", -1.5);
    cvars.init_f32("weapon.thrust_rotation_z", 1.55);
    cvars.init_f32("weapon.thrust_rotation_y", 0.1);
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
        Mesh3d(plane_mesh.clone()),
        MeshMaterial3d(plane_material2.clone()),
        // Rotate 90 degrees around X to make it XY plane (facing Z)
        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2))
            .with_translation(Vec3::new(256.0, 256.0, 0.0)),
        GroundPlane,
    ));

    commands.spawn((
        Mesh3d(plane_mesh.clone()),
        MeshMaterial3d(plane_material2.clone()),
        Transform::from_rotation(Quat::from_rotation_x(3.0 * FRAC_PI_2))
            .with_translation(Vec3::new(256.0, 256.0, 16.0)),
    ));

    // Load map from data/map.yaml
    let map_yaml = std::fs::read_to_string("data/map.yaml").expect("Failed to read data/map.yaml");
    let map_file: MapFile = serde_yaml::from_str(&map_yaml).expect("Failed to parse map.yaml");
    let lines = map_file.map.grid;

    // Load item definitions from data/item_definitions.yaml
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
    let actor_filename = std::env::var("REPO_ROOT")
        .map(|repo_root| format!("{}/source/assets/base/actors/actors.yaml", repo_root))
        .unwrap_or_else(|_| "data/actor_definitions.yaml".to_string());
    let actor_defs_yaml = std::fs::read_to_string(&actor_filename)
        .unwrap_or_else(|_| panic!("Failed to read {}", actor_filename));
    let actor_defs_file: ActorDefinitionsFile = serde_yaml::from_str(&actor_defs_yaml)
        .unwrap_or_else(|_| panic!("Failed to parse {}", actor_filename));
    let actor_definitions = ActorDefinitions {
        actors: actor_defs_file.actors,
    };

    // Build collision map
    let height = lines.len();
    let width = lines.iter().map(|l| l.len()).max().unwrap_or(0);

    let mut collision_grid = HashMap::new();

    let wall_material = materials.add(StandardMaterial {
        base_color_texture: Some(load_image_texture(
            &asset_server,
            "base/textures/stone_2.png",
        )),
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        metallic: 0.0,
        reflectance: 0.0,
        uv_transform: bevy::math::Affine2::from_scale(Vec2::new(1.0, 1.0)),
        ..default()
    });

    // Add some 8x8x8 cubes that will be used as the walls in the map
    // Translate the mesh by +4.0 in Z so cubes sit on the ground plane
    let cube_mesh = meshes.add(
        Cuboid::new(8.0, 8.0, 8.0)
            .mesh()
            .build()
            .translated_by(Vec3::new(4.0, 4.0, 4.0)),
    );

    let cube_mesh2 = meshes.add(
        Cuboid::new(8.0, 8.0, 16.0)
            .mesh()
            .build()
            .translated_by(Vec3::new(4.0, 4.0, 8.0)),
    );

    // Parse the map and create cubes for each 'X'
    for (row, line) in lines.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            // Mark filled cells in collision grid
            let is_solid = matches!(ch, 'X' | 'x');
            if is_solid {
                collision_grid.insert((col as i32, row as i32), true);
            }

            // Position: each cell is 8x8, so multiply by 8
            let x = col as f32 * 8.0;
            let y = row as f32 * 8.0;

            match ch {
                'X' => {
                    commands.spawn((
                        Mesh3d(cube_mesh2.clone()),
                        MeshMaterial3d(wall_material.clone()),
                        Transform::from_translation(Vec3::new(x, y, 0.0)),
                    ));
                }
                'x' => {
                    commands.spawn((
                        Mesh3d(cube_mesh.clone()),
                        MeshMaterial3d(wall_material.clone()),
                        Transform::from_translation(Vec3::new(x, y, 0.0)),
                    ));
                }
                _ => {}
            }
        }
    }

    // Insert collision map as a resource
    commands.insert_resource(CollisionMap::new(collision_grid, width, height));

    // Initialize item tracker and spawn existing items
    let mut item_tracker = ItemTracker::default();

    for item_pos in &map_file.map.items {
        // Track the item position
        let grid_x = (item_pos.x / 8.0).floor() as i32;
        let grid_y = (item_pos.y / 8.0).floor() as i32;
        item_tracker.positions.insert((grid_x, grid_y));
        item_tracker
            .world_positions
            .push((item_pos.x, item_pos.y, item_pos.item_type.clone()));

        // Get scale from item definition for positioning
        let item_def = item_definitions
            .items
            .get(&item_pos.item_type)
            .unwrap_or_else(|| panic!("Item definition not found: {}", item_pos.item_type));
        let scale = item_def.scale;

        // Spawn the item billboard
        spawn_item(
            &mut commands,
            &mut meshes,
            &mut materials,
            &asset_server,
            &item_definitions.items,
            Vec3::new(item_pos.x, item_pos.y, scale),
            &item_pos.item_type,
        );
    }

    commands.insert_resource(item_tracker);
    commands.insert_resource(item_definitions);

    // Spawn actors from map
    for actor_pos in &map_file.map.actors {
        spawn_actor(
            &mut commands,
            &mut meshes,
            &mut materials,
            &asset_server,
            &actor_definitions.actors,
            Vec3::new(actor_pos.x, actor_pos.y, 0.0), // Z will be set by spawn_actor based on scale
            &actor_pos.actor_type,
        );
    }

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

    initialize_weapon_cvars(&mut cvars);

    // Spawn weapon sprite as child of camera for first-person view
    spawn_weapon_sprite(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        camera_entity,
        &cvars,
    );
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
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    toolbar: Res<Toolbar>,
    console_state: Res<ConsoleState>,
    cvars: Res<CVarRegistry>,
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

        // Check for attack input (left mouse button) - only swing if slot 1 is active
        if mouse_button.just_pressed(MouseButton::Left)
            && !weapon.is_swinging
            && toolbar.active_slot == 1
            && !console_state.visible
        {
            // Check if any UI element is being interacted with
            let ui_blocked = ui_interaction_query
                .iter()
                .any(|interaction| *interaction != Interaction::None);
            if !ui_blocked {
                weapon.is_swinging = true;
                weapon.swing_timer.reset();
                weapon.collision_checked = false; // Reset collision check for new swing
            }
        }

        if weapon.is_swinging {
            weapon.swing_timer.tick(time.delta());
            let progress = weapon.swing_timer.fraction();

            // Read animation parameters from cvars
            let rest_pos = Vec3::new(
                cvars.get_f32("weapon.rest_pos_x"),
                cvars.get_f32("weapon.rest_pos_y"),
                cvars.get_f32("weapon.rest_pos_z"),
            );
            let rest_rotation_z = cvars.get_f32("weapon.rest_rotation_z");
            let rest_rotation_y = cvars.get_f32("weapon.rest_rotation_y");

            let windup_pos = Vec3::new(
                cvars.get_f32("weapon.windup_pos_x"),
                cvars.get_f32("weapon.windup_pos_y"),
                cvars.get_f32("weapon.windup_pos_z"),
            );
            let windup_rotation_z = cvars.get_f32("weapon.windup_rotation_z");
            let windup_rotation_y = cvars.get_f32("weapon.windup_rotation_y");

            let thrust_pos = Vec3::new(
                cvars.get_f32("weapon.thrust_pos_x"),
                cvars.get_f32("weapon.thrust_pos_y"),
                cvars.get_f32("weapon.thrust_pos_z"),
            );
            let thrust_rotation_z = cvars.get_f32("weapon.thrust_rotation_z");
            let thrust_rotation_y = cvars.get_f32("weapon.thrust_rotation_y");

            let windup_end = cvars.get_f32("weapon.windup_end");
            let swing_end = cvars.get_f32("weapon.swing_end");

            // Calculate current position and rotation based on phase
            let (current_pos, current_rotation_z, current_rotation_y) = if progress < windup_end {
                // Wind-up phase: pull back toward camera
                let phase_t = progress / windup_end;
                let eased_t = ease_out_quad(phase_t);

                (
                    rest_pos.lerp(windup_pos, eased_t),
                    rest_rotation_z + (windup_rotation_z - rest_rotation_z) * eased_t,
                    rest_rotation_y + (windup_rotation_y - rest_rotation_y) * eased_t,
                )
            } else if progress < swing_end {
                // Thrust phase: fast FORWARD motion with rotation
                let phase_t = (progress - windup_end) / (swing_end - windup_end);
                let eased_t = ease_in_out_cubic(phase_t);

                (
                    windup_pos.lerp(thrust_pos, eased_t),
                    windup_rotation_z + (thrust_rotation_z - windup_rotation_z) * eased_t,
                    windup_rotation_y + (thrust_rotation_y - windup_rotation_y) * eased_t,
                )
            } else {
                // Follow-through phase: deceleration back to rest
                let phase_t = (progress - swing_end)
                    / (cvars.get_f32("weapon.followthrough_end") - swing_end);
                let eased_t = ease_out_quad(phase_t);

                (
                    thrust_pos.lerp(rest_pos, eased_t),
                    thrust_rotation_z + (rest_rotation_z - thrust_rotation_z) * eased_t,
                    thrust_rotation_y + (rest_rotation_y - thrust_rotation_y) * eased_t,
                )
            };

            // Apply transforms - combine Z rotation and Y rotation (tilt)
            transform.translation = current_pos;
            transform.rotation = Quat::from_rotation_z(current_rotation_z)
                * Quat::from_rotation_y(current_rotation_y);
            // Check if animation is complete
            if weapon.swing_timer.is_finished() {
                weapon.is_swinging = false;
                weapon.collision_checked = false;
                transform.translation = rest_pos;
                transform.rotation =
                    Quat::from_rotation_z(rest_rotation_z) * Quat::from_rotation_y(rest_rotation_y);
            }
        }
    }
}

fn update_weapon_swing_collision(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut actor_query: Query<(Entity, &Transform, &mut Actor), (With<Billboard>, Without<Item>)>,
    mut weapon_query: Query<&mut WeaponSprite>,
    mut cvars: ResMut<CVarRegistry>,
    mut stats: ResMut<PlayerStats>,
    actor_definitions: Res<ActorDefinitions>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    for mut weapon in weapon_query.iter_mut() {
        // Only check collision once during the swing phase
        if !weapon.is_swinging || weapon.collision_checked {
            continue;
        }

        let progress = weapon.swing_timer.fraction();
        let windup_end = cvars.get_f32("weapon.windup_end");
        let swing_end = cvars.get_f32("weapon.swing_end");

        // Check collision during the thrust phase (when weapon is extended)
        // We'll check at about 50% through the thrust phase for best timing
        let thrust_check_point = windup_end + (swing_end - windup_end) * 0.5;

        if progress >= thrust_check_point && !weapon.collision_checked {
            weapon.collision_checked = true;

            // Get camera position and forward direction
            let camera_pos = camera_transform.translation;
            let forward = camera_transform.forward().as_vec3();

            // Project forward direction to XY plane and normalize
            let forward_xy = Vec2::new(forward.x, forward.y).normalize_or_zero();

            // Define collision box in front of player
            let check_distance = 8.0; // How far in front to check
            let check_width = 4.0; // Width of the collision box (half-width on each side)

            // Calculate right vector perpendicular to forward (for width check)
            let right_xy = Vec2::new(-forward_xy.y, forward_xy.x);

            // Check all actors (excluding items)
            let mut hit_any = false;

            for (_entity, actor_transform, mut actor) in actor_query.iter_mut() {
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
                if forward_distance > check_distance + 4.0 {
                    continue;
                }

                if forward_distance < check_distance - 4.0 {
                    continue;
                }

                // Width check: project onto right vector to get lateral distance
                let lateral_distance = to_actor.dot(right_xy).abs();

                if lateral_distance <= check_width {
                    hit_any = true;

                    // Get the actor definition to run the on_hit script
                    if let Some(actor_def) = actor_definitions.actors.get(&actor.actor_type) {
                        if !actor_def.on_hit.is_empty() {
                            let output = scripting::process_script_with_actor(
                                &actor_def.on_hit,
                                &mut stats,
                                &mut cvars,
                                Some(&mut *actor),
                            );
                            for line in &output {
                                println!("{}", line);
                            }
                        }
                    }

                    break;
                }
            }

            // Print collision status
            if hit_any {
                println!("collision!");
            } else {
                println!("no collision");
            }
        }
    }
}

fn spawn_billboard_sprite(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    sprite_path: &str,
    scale: f32,
) {
    let sprite_material = materials.add(StandardMaterial {
        base_color_texture: Some(load_image_texture(asset_server, sprite_path)),
        base_color: Color::WHITE,
        alpha_mode: bevy::render::alpha::AlphaMode::Blend,
        unlit: false,
        cull_mode: None,
        ..default()
    });

    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};

    let mut billboard_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    let positions = vec![
        [0.0, -scale, -scale], // bottom-left
        [0.0, scale, -scale],  // top-left
        [0.0, scale, scale],   // top-right
        [0.0, -scale, scale],  // bottom-right
    ];
    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[1.0, 0.0, 0.0]; 4]);

    let uvs = vec![
        [0.0, 1.0], // top-left -> bottom-left in texture
        [1.0, 1.0], // top-right -> bottom-right in texture
        [1.0, 0.0], // bottom-right -> top-right in texture
        [0.0, 0.0], // bottom-left -> top-left in texture
    ];
    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    billboard_mesh.insert_indices(Indices::U32(vec![
        0, 1, 2, // first triangle
        0, 2, 3, // second triangle
    ]));

    commands.spawn((
        Mesh3d(meshes.add(billboard_mesh)),
        MeshMaterial3d(sprite_material),
        Transform::from_translation(position),
        Billboard,
    ));
}

fn spawn_weapon_sprite(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    camera_entity: Entity,
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

    // Spawn weapon as child of camera
    // Position it to the right and lower on screen
    // Close to camera to ensure it renders on top
    let rest_pos = Vec3::new(
        cvars.get_f32("weapon.rest_pos_x"),
        cvars.get_f32("weapon.rest_pos_y"),
        cvars.get_f32("weapon.rest_pos_z"),
    );

    let weapon_entity = commands
        .spawn((
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

fn spawn_item(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    item_definitions: &HashMap<String, ItemDefinition>,
    position: Vec3,
    item_key: &str,
) {
    let item_def = item_definitions
        .get(item_key)
        .unwrap_or_else(|| panic!("Item definition not found: {}", item_key));

    let sprite_material = materials.add(StandardMaterial {
        base_color_texture: Some(load_image_texture(asset_server, &item_def.image)),
        base_color: Color::WHITE,
        alpha_mode: bevy::render::alpha::AlphaMode::Blend,
        unlit: false,
        cull_mode: None,
        ..default()
    });

    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};

    let scale = item_def.scale;

    let mut billboard_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    let positions = vec![
        [0.0, -scale, -scale],
        [0.0, scale, -scale],
        [0.0, scale, scale],
        [0.0, -scale, scale],
    ];
    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[1.0, 0.0, 0.0]; 4]);

    let uvs = vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    billboard_mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));

    commands.spawn((
        Mesh3d(meshes.add(billboard_mesh)),
        MeshMaterial3d(sprite_material),
        Transform::from_translation(position),
        Billboard,
        Item {
            interaction_radius: 2.0,
        },
    ));
}

fn spawn_actor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    actor_definitions: &HashMap<String, ActorDefinition>,
    position: Vec3,
    actor_key: &str,
) {
    let actor_def = actor_definitions
        .get(actor_key)
        .unwrap_or_else(|| panic!("Actor definition not found: {}", actor_key));

    let sprite_material = materials.add(StandardMaterial {
        base_color_texture: Some(load_image_texture(asset_server, &actor_def.sprite)),
        base_color: Color::WHITE,
        alpha_mode: bevy::render::alpha::AlphaMode::Blend,
        unlit: false,
        cull_mode: None,
        ..default()
    });

    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};

    let scale = actor_def.scale;

    let mut billboard_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    let positions = vec![
        [0.0, -scale, -scale],
        [0.0, scale, -scale],
        [0.0, scale, scale],
        [0.0, -scale, scale],
    ];
    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[1.0, 0.0, 0.0]; 4]);

    let uvs = vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    billboard_mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));

    commands.spawn((
        Mesh3d(meshes.add(billboard_mesh)),
        MeshMaterial3d(sprite_material),
        Transform::from_translation(Vec3::new(position.x, position.y, scale)),
        Billboard,
        Actor {
            actor_type: actor_key.to_string(),
            health: actor_def.max_health,
            max_health: actor_def.max_health,
            scale: actor_def.scale,
        },
    ));
}

fn update_spawn_item_on_click(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&bevy::window::Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    ground_query: Query<&GlobalTransform, With<GroundPlane>>,
    ui_interaction_query: Query<&Interaction>,
    mut item_tracker: ResMut<ItemTracker>,
    toolbar: Res<Toolbar>,
    item_definitions: Res<ItemDefinitions>,
    console_state: Res<ConsoleState>,
) {
    // Don't spawn items if console is open
    if console_state.visible {
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Left) {
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
    if item_tracker.positions.contains(&(grid_x, grid_y)) {
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

    // Track the item
    item_tracker.positions.insert((grid_x, grid_y));
    item_tracker
        .world_positions
        .push((world_x, world_y, item_key.to_string()));

    // Get scale from item definition for positioning
    let item_def = item_definitions
        .items
        .get(item_key)
        .expect("Item definition not found");
    let scale = item_def.scale;

    // Spawn item billboard at the intersection point
    spawn_item(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        &item_definitions.items,
        Vec3::new(world_x, world_y, scale),
        item_key,
    );
}

fn update_save_map_on_input(
    input: Res<ButtonInput<KeyCode>>,
    item_tracker: Res<ItemTracker>,
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
        // Read the current map file
        let map_yaml = match std::fs::read_to_string("data/map.yaml") {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read map.yaml: {}", e);
                return;
            }
        };

        let mut map_file: MapFile = match serde_yaml::from_str(&map_yaml) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to parse map.yaml: {}", e);
                return;
            }
        };

        // Update items in the map data
        map_file.map.items = item_tracker
            .world_positions
            .iter()
            .map(|(x, y, item_type)| ItemPosition {
                x: *x,
                y: *y,
                item_type: item_type.clone(),
            })
            .collect();

        // Serialize and save
        let yaml_output = match serde_yaml::to_string(&map_file) {
            Ok(yaml) => yaml,
            Err(e) => {
                eprintln!("Failed to serialize map: {}", e);
                return;
            }
        };

        if let Err(e) = std::fs::write("data/map.yaml", yaml_output) {
            eprintln!("Failed to write map.yaml: {}", e);
        } else {
            println!(
                "Map saved successfully with {} items!",
                item_tracker.world_positions.len()
            );
        }
    }
}

fn update_actor_death(
    mut commands: Commands,
    actor_query: Query<(Entity, &Actor)>,
    mut stats: ResMut<PlayerStats>,
    mut cvars: ResMut<CVarRegistry>,
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
    mut item_tracker: ResMut<ItemTracker>,
    item_definitions: Res<ItemDefinitions>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (entity, item_transform, item) in item_query.iter() {
        let item_pos = item_transform.translation;

        if check_circle_collision(player_pos, item_pos, item.interaction_radius) {
            // Find the item type from the tracker
            let item_type = item_tracker
                .world_positions
                .iter()
                .find(|(x, y, _)| (*x - item_pos.x).abs() < 0.1 && (*y - item_pos.y).abs() < 0.1)
                .map(|(_, _, item_type)| item_type.as_str())
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

            // Remove from tracker
            item_tracker.remove_at_position(item_pos.x, item_pos.y);

            println!("Collected item! Fatigue: {}", stats.stamina);
        }
    }
}
