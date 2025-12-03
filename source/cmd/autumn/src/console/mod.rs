//! Console Module
//!
//! Registers "Quake-like" Console for during GameState::Playing for issuing
//! commands and setting runtime variables.
//!
//! Includes the UI for the console.

mod console_plugin;
mod console_state;
mod console_ui;

// Bevy Plugins
pub use console_plugin::ConsolePlugin;

// Bevy Components & Resources
pub use console_state::ConsoleState;

//
// internal mod
//
pub mod internal {
    pub const MAX_HISTORY_LINES: usize = 200;

    pub use crate::game_state::{GamePlayEntity, GameState};

    pub use bevy::prelude::*;
}
