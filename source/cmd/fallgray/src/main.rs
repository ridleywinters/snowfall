mod ui;

use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use ui::*;

const PLAYER_LIGHT_OFFSET_1: [f32; 3] = [0.0, 1.5, 4.0];
const PLAYER_LIGHT_OFFSET_2: [f32; 3] = [0.5, -0.5, 4.0];
const PLAYER_RADIUS: f32 = 8.0 * 0.2;

#[derive(Deserialize)]
struct MapFile {
    map: MapData,
}

#[derive(Deserialize)]
struct MapData {
    grid: Vec<String>,
}

#[derive(Resource)]
struct CollisionMap {
    grid: HashMap<(i32, i32), bool>,
    width: usize,
    height: usize,
}

impl CollisionMap {
    fn is_solid(&self, grid_x: i32, grid_y: i32) -> bool {
        // Check bounds first
        if grid_x < 0 || grid_y < 0 || grid_x >= self.width as i32 || grid_y >= self.height as i32 {
            return true; // Treat out of bounds as solid
        }
        // If not in map, treat as empty (false)
        *self.grid.get(&(grid_x, grid_y)).unwrap_or(&false)
    }

    fn can_move_to(&self, world_x: f32, world_y: f32, radius: f32) -> bool {
        // Treat player as a 2D bounding box with width and height of 2 * radius
        let half_size = radius;
        let min_x = world_x - half_size;
        let max_x = world_x + half_size;
        let min_y = world_y - half_size;
        let max_y = world_y + half_size;

        // Calculate grid cell range that the bounding box overlaps
        // Check all cells that any part of the box could touch
        let min_grid_x = (min_x / 8.0).floor() as i32;
        let max_grid_x = (max_x / 8.0).floor() as i32;
        let min_grid_y = (min_y / 8.0).floor() as i32;
        let max_grid_y = (max_y / 8.0).floor() as i32;

        // Check if any of the cells the bounding box overlaps is solid
        for grid_y in min_grid_y..=max_grid_y {
            for grid_x in min_grid_x..=max_grid_x {
                let solid = self.is_solid(grid_x, grid_y);
                if solid {
                    return false;
                }
            }
        }

        true
    }
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
        .add_systems(Startup, setup_system)
        .add_systems(Startup, setup_ui)
        .add_systems(
            Update,
            (
                camera_control_system,
                update_player_light,
                animate_player_light,
                update_ui,
                test_stats_input,
                update_billboards,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player {
    speed: f32,
    rot_speed: f32,
}

#[derive(Component)]
struct PlayerLight;

#[derive(Component)]
struct PlayerLight2;

#[derive(Component)]
struct Billboard;

#[derive(Component)]
struct LightColorAnimation {
    time: f32,
    speed: f32,
}

impl Default for LightColorAnimation {
    fn default() -> Self {
        Self {
            time: 0.0,
            speed: 1.0,
        }
    }
}

fn load_image_texture<T: Into<String>>(asset_server: &Res<AssetServer>, path: T) -> Handle<Image> {
    asset_server.load_with_settings(
        path.into(),
        |settings: &mut bevy::image::ImageLoaderSettings| {
            settings.sampler =
                bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                    address_mode_u: bevy::image::ImageAddressMode::Repeat,
                    address_mode_v: bevy::image::ImageAddressMode::Repeat,
                    mag_filter: bevy::image::ImageFilterMode::Nearest,
                    min_filter: bevy::image::ImageFilterMode::Nearest,
                    ..Default::default()
                });
        },
    )
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
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
    ));

    commands.spawn((
        Mesh3d(plane_mesh.clone()),
        MeshMaterial3d(plane_material2.clone()),
        Transform::from_rotation(Quat::from_rotation_x(3.0 * FRAC_PI_2))
            .with_translation(Vec3::new(256.0, 256.0, 16.0)),
    ));

    // Add some 8x8x8 cubes as reference points
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

    // Load map from data/map.yaml
    let map_yaml = std::fs::read_to_string("data/map.yaml").expect("Failed to read data/map.yaml");
    let map_file: MapFile = serde_yaml::from_str(&map_yaml).expect("Failed to parse map.yaml");
    let lines = map_file.map.grid;

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
                'c' => {
                    // Spawn a billboarded sprite
                    let sprite_material = materials.add(StandardMaterial {
                        base_color_texture: Some(load_image_texture(
                            &asset_server,
                            "base/sprites/npc-0001.png",
                        )),
                        base_color: Color::WHITE,
                        alpha_mode: bevy::render::alpha::AlphaMode::Blend,
                        unlit: false,
                        cull_mode: None, // Render both sides
                        ..default()
                    });

                    // Create a vertical plane mesh with flipped UVs
                    // Normal pointing in X direction makes it vertical (parallel to YZ plane)
                    use bevy::asset::RenderAssetUsages;
                    use bevy::mesh::{Indices, PrimitiveTopology};

                    let mut billboard_mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::default(),
                    );

                    let scale = 3.8;
                    let positions = vec![
                        [0.0, -scale, -scale], // bottom-left
                        [0.0, scale, -scale],  // top-left
                        [0.0, scale, scale],   // top-right
                        [0.0, -scale, scale],  // bottom-right
                    ];
                    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

                    // Normals pointing in +X direction
                    billboard_mesh
                        .insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[1.0, 0.0, 0.0]; 4]);

                    // UV coordinates (flipped in Y: 1.0 - original V)
                    let uvs = vec![
                        [0.0, 1.0], // top-left -> bottom-left in texture
                        [1.0, 1.0], // top-right -> bottom-right in texture
                        [1.0, 0.0], // bottom-right -> top-right in texture
                        [0.0, 0.0], // bottom-left -> top-left in texture
                    ];
                    billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

                    // Indices for two triangles
                    billboard_mesh.insert_indices(Indices::U32(vec![
                        0, 1, 2, // first triangle
                        0, 2, 3, // second triangle
                    ]));

                    commands.spawn((
                        Mesh3d(meshes.add(billboard_mesh)),
                        MeshMaterial3d(sprite_material),
                        Transform::from_translation(Vec3::new(x + 4.0, y + 4.0, scale)),
                        Billboard,
                    ));
                }
                _ => {}
            }
        }
    }

    // Insert collision map as a resource
    commands.insert_resource(CollisionMap {
        grid: collision_grid,
        width,
        height,
    });

    commands.insert_resource(bevy::light::AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
        affects_lightmapped_meshes: false,
    });

    let player_start_pos = Vec3::new(256.0 + 4.0, 200.0 + 4.0, 4.8);

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(player_start_pos.x, player_start_pos.y, player_start_pos.z).looking_at(
            Vec3::new(
                player_start_pos.x - 1.0,
                player_start_pos.y,
                player_start_pos.z * 1.01,
            ),
            Vec3::Z,
        ),
        Player {
            speed: 32.0,
            rot_speed: 2.75,
        },
    ));

    // Add a point light that follows the player
    commands.spawn((
        PointLight {
            color: Color::WHITE,
            intensity: 1000000.0,
            range: 64.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(
            player_start_pos.x + PLAYER_LIGHT_OFFSET_1[0],
            player_start_pos.y + PLAYER_LIGHT_OFFSET_1[1],
            player_start_pos.z + PLAYER_LIGHT_OFFSET_1[2],
        ),
        PlayerLight, // Marker component to identify this light
        LightColorAnimation::default(),
    ));

    // Add a second point light that follows the player with no Y offset
    commands.spawn((
        PointLight {
            color: Color::WHITE,
            intensity: 1000000.0,
            range: 64.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(
            player_start_pos.x + PLAYER_LIGHT_OFFSET_2[0],
            player_start_pos.y + PLAYER_LIGHT_OFFSET_2[1],
            player_start_pos.z + PLAYER_LIGHT_OFFSET_2[2],
        ),
        PlayerLight2,
    ));
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

fn camera_control_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    collision_map: Res<CollisionMap>,
    mut query: Query<(&mut Transform, &Player)>,
) {
    for (mut transform, player) in query.iter_mut() {
        let dt = time.delta_secs();

        // Movement input (WASD + QE)
        // WASD moves in the XY plane, Q/E moves along Z axis
        let mut movement_xy = Vec2::ZERO; // Movement in XY plane
        let mut movement_z = 0.0; // Movement along Z axis

        if input.pressed(KeyCode::KeyW) {
            movement_xy.y += 1.0; // Forward
        }
        if input.pressed(KeyCode::KeyS) {
            movement_xy.y -= 1.0; // Backward
        }
        if input.pressed(KeyCode::KeyA) {
            movement_xy.x -= 1.0; // Left
        }
        if input.pressed(KeyCode::KeyD) {
            movement_xy.x += 1.0; // Right
        }
        if input.pressed(KeyCode::KeyF) {
            movement_z -= 1.0; // Down
        }
        if input.pressed(KeyCode::KeyR) {
            movement_z += 1.0; // Up
        }

        // Rotation input (Arrow keys)
        // Arrow left/right rotates around Z axis (yaw)
        // Arrow up/down changes pitch (looking up/down)
        let mut yaw_delta = 0.0;
        let mut pitch_delta = 0.0;

        if input.pressed(KeyCode::ArrowLeft) {
            yaw_delta += player.rot_speed * dt;
        }
        if input.pressed(KeyCode::ArrowRight) {
            yaw_delta -= player.rot_speed * dt;
        }
        if input.pressed(KeyCode::ArrowUp) {
            pitch_delta += player.rot_speed * dt;
        }
        if input.pressed(KeyCode::ArrowDown) {
            pitch_delta -= player.rot_speed * dt;
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

            let max = scale * player.rot_speed * dt;
            yaw_delta += (yaw_snap - yaw).clamp(-max, max);
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

                // Calculate new pitch and clamp to limits
                let pitch_limit = 70_f32.to_radians();
                let new_pitch = (current_pitch + pitch_delta).clamp(-pitch_limit, pitch_limit);
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
            if collision_map.can_move_to(new_x, new_y, PLAYER_RADIUS) {
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

#[allow(clippy::type_complexity)]
fn update_player_light(
    player_query: Query<&Transform, With<Player>>,
    mut light_query: Query<&mut Transform, (With<PlayerLight>, Without<Player>)>,
    mut light2_query: Query<
        &mut Transform,
        (With<PlayerLight2>, Without<Player>, Without<PlayerLight>),
    >,
) {
    if let Ok(player_transform) = player_query.single() {
        // Update first light with Y offset
        if let Ok(mut light_transform) = light_query.single_mut() {
            light_transform.translation = player_transform.translation
                + Vec3::new(
                    PLAYER_LIGHT_OFFSET_1[0],
                    PLAYER_LIGHT_OFFSET_1[1],
                    PLAYER_LIGHT_OFFSET_1[2],
                );
        }

        // Update second light with no Y offset
        if let Ok(mut light2_transform) = light2_query.single_mut() {
            light2_transform.translation = player_transform.translation
                + Vec3::new(
                    PLAYER_LIGHT_OFFSET_2[0],
                    PLAYER_LIGHT_OFFSET_2[1],
                    PLAYER_LIGHT_OFFSET_2[2],
                );
        }
    }
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;

    Color::srgb(r, g, b)
}

fn animate_player_light(
    time: Res<Time>,
    mut light_query: Query<(&mut PointLight, &mut LightColorAnimation), With<PlayerLight>>,
) {
    if let Ok((mut light, mut anim)) = light_query.single_mut() {
        let dt = time.delta_secs();
        anim.time += 0.1 * dt * anim.speed;

        let light_yellow = hex_to_color("#e8d599");
        let red = hex_to_color("#ffb96e");
        let yellow_white = hex_to_color("#e4bb6f");

        // Create a smooth oscillation through the three colors
        // Use sine wave that goes 0 -> 1 -> 2 -> 1 -> 0 (one full cycle)
        let t = (anim.time * std::f32::consts::PI).sin().abs();

        // Map t (0 to 1) to blend between the three colors
        let color = if t < 0.5 {
            // Blend from light_yellow to red
            let blend = t * 2.0; // 0 to 1
            Color::srgb(
                light_yellow.to_srgba().red * (1.0 - blend) + red.to_srgba().red * blend,
                light_yellow.to_srgba().green * (1.0 - blend) + red.to_srgba().green * blend,
                light_yellow.to_srgba().blue * (1.0 - blend) + red.to_srgba().blue * blend,
            )
        } else {
            // Blend from red to yellow_white
            let blend = (t - 0.5) * 2.0; // 0 to 1
            Color::srgb(
                red.to_srgba().red * (1.0 - blend) + yellow_white.to_srgba().red * blend,
                red.to_srgba().green * (1.0 - blend) + yellow_white.to_srgba().green * blend,
                red.to_srgba().blue * (1.0 - blend) + yellow_white.to_srgba().blue * blend,
            )
        };

        light.color = color;

        // When we complete a cycle, randomize the speed for next cycle (+/- 20%)
        if anim.time >= 2.0 {
            anim.time = 0.0;
            let mut rng = rand::rng();
            anim.speed = 1.0 + rng.random_range(-0.2..0.2);
        }
    }
}
