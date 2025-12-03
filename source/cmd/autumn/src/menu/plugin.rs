use bevy::prelude::*;

use super::ui::*;
use crate::game_state::GameState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Main Menu systems
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
            // Game Over systems
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over)
            // Menu button systems (active in all menu states)
            .add_systems(
                Update,
                (handle_menu_buttons, update_button_visuals)
                    .run_if(not(in_state(GameState::Playing))),
            );
    }
}
