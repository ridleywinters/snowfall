use bevy::prelude::*;

/// Game state enum for menu and gameplay transitions
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    GameOver,
}

/// Marker for entities that should be cleaned up when leaving Playing state
#[derive(Component)]
pub struct GamePlayEntity;
