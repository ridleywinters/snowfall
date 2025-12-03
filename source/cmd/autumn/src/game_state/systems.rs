use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::camera::{MouseLookSettings, Player};

use super::states::*;

/// System to detect player death and transition to game over
pub fn detect_player_death(
    player_query: Query<&Player>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *current_state.get() != GameState::Playing {
        return;
    }

    if let Ok(player) = player_query.single() {
        if !player.is_alive() {
            info!("Player died! Transitioning to GameOver state");
            next_state.set(GameState::GameOver);
        }
    }
}

/// Clean up game entities when leaving Playing state
pub fn cleanup_game_entities(
    mut commands: Commands,
    entities: Query<Entity, With<GamePlayEntity>>,
) {
    info!(
        "Cleaning up game entities (found {} entities)",
        entities.iter().count()
    );
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}

/// Unlock cursor when entering menu states (MainMenu or GameOver)
pub fn unlock_cursor_on_menu(
    mut mouse_look: ResMut<MouseLookSettings>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // Unlock cursor and make it visible
    mouse_look.cursor_locked = false;

    if let Ok(mut cursor) = cursor_query.single_mut() {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}
