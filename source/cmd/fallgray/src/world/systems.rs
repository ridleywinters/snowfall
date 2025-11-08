use bevy::prelude::*;
use crate::actor::{ActorDefinitions, ActorDefinitionsFile};
use crate::camera::{spawn_camera, spawn_player_lights};
use crate::combat::{CombatAudio, WeaponDefinitions};
use crate::game_state::GamePlayEntity;
use crate::item::{ItemDefinitions, ItemDefinitionsFile};
use crate::map::Map;
use crate::scripting::CVarRegistry;
use crate::texture_loader::load_image_texture;
use crate::weapon::spawn_weapon_sprite;
use std::f32::consts::FRAC_PI_2;

#[derive(Component)]
pub struct GroundPlane;

/// System to set up the game world when entering Playing state
pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut cvars: ResMut<CVarRegistry>,
) {
    // Create ground planes
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(512.0, 512.0));
    let plane_material = materials.add(StandardMaterial {
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
        MeshMaterial3d(plane_material.clone()),
        // Rotate 90 degrees around X to make it XY plane (facing Z)
        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2))
            .with_translation(Vec3::new(256.0, 256.0, 0.0)),
        GroundPlane,
    ));

    commands.spawn((
        GamePlayEntity,
        Mesh3d(plane_mesh.clone()),
        MeshMaterial3d(plane_material.clone()),
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
    spawn_weapon_sprite(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        camera_entity,
        &weapon_definitions,
        &cvars,
    );

    // Insert weapon definitions as resource
    commands.insert_resource(weapon_definitions);

    // Load combat audio
    let combat_audio = CombatAudio::load_sounds(&asset_server);
    commands.insert_resource(combat_audio);
}
