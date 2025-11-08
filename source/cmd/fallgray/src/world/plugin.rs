use bevy::prelude::*;
use crate::game_state::GameState;
use super::systems::setup_world;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_world);
    }
}
