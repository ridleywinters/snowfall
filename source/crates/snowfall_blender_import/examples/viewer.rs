use anyhow::{Context, Result};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::{Cursor, CursorGrabMode};
use bevy_flycam::prelude::*;
use snowfall_blender_import::{MNode, load_from_file};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <blender_file.blend>", args[0]);
        std::process::exit(1);
    }

    let blend_path = &args[1];
    println!("Loading blend file: {}", blend_path);

    let blend_file = load_from_file(blend_path)
        .with_context(|| format!("Failed to load blend file: {}", blend_path))?;

    println!("Loaded Blender {} file", blend_file.version_string());
    println!("Meshes: {}", blend_file.scene.meshes.len());

    let scene_bounds = blend_file.scene.scene_bounds();
    let center = scene_bounds.center();
    let radius = scene_bounds.sphere_radius();

    println!("Scene bounds: {:?}", scene_bounds);
    println!("Scene center: {:?}", center);
    println!("Scene radius: {}", radius);

    let movement_speed = (radius * 1.5).max(1.0);
    println!("Movement speed: {}", movement_speed);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Blender File Viewer".to_string(),
                cursor: Cursor {
                    visible: false,
                    grab_mode: CursorGrabMode::Locked,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(NoCameraPlayerPlugin)
        .insert_resource(BlendScene {
            scene: blend_file.scene,
            center: Vec3::new(center.x, center.y, center.z),
            radius,
        })
        .insert_resource(MovementSettings {
            sensitivity: 0.00012,
            speed: movement_speed,
        })
        .insert_resource(KeyBindings {
            move_ascend: KeyCode::KeyR,
            move_descend: KeyCode::KeyF,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_cursor_grab)
        .run();

    Ok(())
}

#[derive(Resource)]
struct BlendScene {
    scene: snowfall_blender_import::MScene,
    center: Vec3,
    radius: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    blend_scene: Res<BlendScene>,
) {
    let checkered_texture = create_checkered_texture();
    let texture_handle = images.add(checkered_texture);

    let checkered_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        metallic: 0.0,
        perceptual_roughness: 0.8,
        ..default()
    });

    spawn_scene_nodes(
        &mut commands,
        &mut meshes,
        &checkered_material,
        &blend_scene.scene,
        &blend_scene.scene.root.children,
        Transform::IDENTITY,
    );

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    let camera_distance = blend_scene.radius * 2.5;
    let camera_pos =
        blend_scene.center + Vec3::new(camera_distance, camera_distance, camera_distance * 0.5);

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(camera_pos)
                .looking_at(blend_scene.center, Vec3::Z),
            ..default()
        },
        FlyCam,
    ));
}

fn spawn_scene_nodes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: &Handle<StandardMaterial>,
    scene: &snowfall_blender_import::MScene,
    nodes: &[MNode],
    parent_transform: Transform,
) {
    for node in nodes {
        match node {
            MNode::MInstance(instance) => {
                if let Some(mesh_data) = scene.meshes.get(&instance.geometry_id) {
                    if mesh_data.positions.is_empty() || mesh_data.indices.is_empty() {
                        continue;
                    }

                    let mut bevy_mesh = Mesh::new(
                        bevy::render::mesh::PrimitiveTopology::TriangleList,
                        bevy::render::render_asset::RenderAssetUsages::default(),
                    );

                    bevy_mesh.insert_attribute(
                        Mesh::ATTRIBUTE_POSITION,
                        mesh_data
                            .positions
                            .iter()
                            .map(|v| [v.x, v.y, v.z])
                            .collect::<Vec<_>>(),
                    );

                    bevy_mesh.insert_indices(bevy::render::mesh::Indices::U32(
                        mesh_data.indices.clone(),
                    ));

                    if mesh_data.normals.len() == mesh_data.positions.len() {
                        bevy_mesh.insert_attribute(
                            Mesh::ATTRIBUTE_NORMAL,
                            mesh_data
                                .normals
                                .iter()
                                .map(|v| [v.x, v.y, v.z])
                                .collect::<Vec<_>>(),
                        );
                    } else {
                        bevy_mesh.compute_normals();
                    }

                    if mesh_data.uvs.len() == mesh_data.positions.len() {
                        bevy_mesh.insert_attribute(
                            Mesh::ATTRIBUTE_UV_0,
                            mesh_data.uvs.iter().map(|v| [v.x, v.y]).collect::<Vec<_>>(),
                        );
                    } else {
                        let uvs: Vec<[f32; 2]> = mesh_data
                            .positions
                            .iter()
                            .map(|pos| [pos.x * 0.1, pos.y * 0.1])
                            .collect();
                        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                    }

                    let mesh_handle = meshes.add(bevy_mesh);

                    let instance_transform = if let Some(t) = &instance.transform {
                        mtransform_to_bevy(t)
                    } else {
                        Transform::IDENTITY
                    };

                    let world_transform = parent_transform * instance_transform;

                    commands.spawn((PbrBundle {
                        mesh: mesh_handle,
                        material: material.clone(),
                        transform: world_transform,
                        ..default()
                    },));
                }
            }
            MNode::MGroup(group) => {
                let group_transform = if let Some(t) = &group.transform {
                    mtransform_to_bevy(t)
                } else {
                    Transform::IDENTITY
                };

                let world_transform = parent_transform * group_transform;

                spawn_scene_nodes(
                    commands,
                    meshes,
                    material,
                    scene,
                    &group.children,
                    world_transform,
                );
            }
        }
    }
}

fn mtransform_to_bevy(t: &snowfall_blender_import::MTransform) -> Transform {
    Transform {
        translation: Vec3::new(t.translation.x, t.translation.y, t.translation.z),
        rotation: Quat::from_euler(EulerRot::XYZ, t.rotation.x, t.rotation.y, t.rotation.z),
        scale: Vec3::new(t.scale.x, t.scale.y, t.scale.z),
    }
}

fn create_checkered_texture() -> Image {
    const SIZE: u32 = 32;
    const LIGHT_GRAY: u8 = 212;
    const DARK_GRAY: u8 = 32;

    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let is_light = (x / 2 + y / 2) % 2 == 0;
            let gray = if is_light { LIGHT_GRAY } else { DARK_GRAY };
            data.extend_from_slice(&[gray, gray, gray, 255]);
        }
    }

    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn toggle_cursor_grab(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Tab) {
        if window.cursor.grab_mode == CursorGrabMode::Locked {
            window.cursor.visible = true;
            window.cursor.grab_mode = CursorGrabMode::None;
        } else {
            window.cursor.visible = false;
            window.cursor.grab_mode = CursorGrabMode::Locked;
        }
    }
}
