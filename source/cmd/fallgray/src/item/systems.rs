use bevy::prelude::*;
use crate::camera::Player;
use crate::collision::check_circle_collision;
use crate::map::Map;
use crate::scripting::{self, CVarRegistry};
use crate::ui::PlayerStats;
use super::components::Item;
use super::definitions::ItemDefinitions;

/// System to check for item collision and pickup
pub fn update_check_item_collision(
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
