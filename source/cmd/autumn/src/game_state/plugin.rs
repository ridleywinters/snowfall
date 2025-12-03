use bevy::prelude::*;

use super::states::GameState;
use super::systems::*;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            // Main Menu systems
            .add_systems(OnEnter(GameState::MainMenu), unlock_cursor_on_menu)
            // Playing state systems
            .add_systems(Update, detect_player_death.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), cleanup_game_entities)
            // Game Over systems
            .add_systems(OnEnter(GameState::GameOver), unlock_cursor_on_menu);
    }
}
