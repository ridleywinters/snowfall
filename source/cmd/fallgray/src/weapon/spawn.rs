use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use crate::combat::WeaponDefinitions;
use crate::game_state::GamePlayEntity;
use crate::scripting::CVarRegistry;
use crate::texture_loader::load_weapon_texture;
use super::components::WeaponSprite;

/// Spawn weapon sprite as child of camera entity
pub fn spawn_weapon_sprite(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    camera_entity: Entity,
    weapon_definitions: &WeaponDefinitions,
    cvars: &CVarRegistry,
) {
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
