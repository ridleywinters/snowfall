use bevy::prelude::*;
use crate::game_state::GameState;
use super::systems::update_check_item_collision;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_check_item_collision.run_if(in_state(GameState::Playing)),
        );
    }
}
