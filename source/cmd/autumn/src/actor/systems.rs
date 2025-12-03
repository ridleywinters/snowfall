use super::components::Actor;
use super::definitions::ActorDefinitions;
use crate::hud::PlayerStats;
use crate::scripting::{self, CVarRegistry};
use crate::world::Map;
use bevy::prelude::*;

/// System to handle actor death and cleanup
pub fn update_actor_death(
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

/// System to update actor health indicators (visual feedback via tinting)
pub fn update_actor_health_indicators(
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
