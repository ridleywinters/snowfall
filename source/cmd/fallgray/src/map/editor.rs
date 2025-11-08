use bevy::prelude::*;
use crate::console::ConsoleState;
use crate::game_state::GameState;
use crate::item::ItemDefinitions;
use crate::toolbar::Toolbar;
use crate::world::GroundPlane;
use super::Map;

/// Debug/Development system to spawn items by clicking on the ground plane
pub fn update_spawn_item_on_click(
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

/// Debug/Development system to save map when Ctrl+S is pressed
pub fn update_save_map_on_input(
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

/// Plugin to register map editor systems
pub struct MapEditorPlugin;

impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_spawn_item_on_click,
                update_save_map_on_input,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
