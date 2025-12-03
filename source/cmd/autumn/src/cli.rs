use bevy::prelude::*;
use clap::Parser;

/// Command-line arguments for the game
#[derive(Parser, Debug, Clone, Resource)]
#[command(name = "fallgray")]
#[command(about = "Fallgray game engine", long_about = None)]
pub struct Args {
    /// Path to a GLTF level file to load
    #[arg(long, value_name = "PATH")]
    pub level: Option<String>,
}

/// Get the level path from command-line arguments if set
pub fn get_level_path(args: &Args) -> Option<&str> {
    // TODO: Process level flag for GLTF map loading
    args.level.as_deref()
}
