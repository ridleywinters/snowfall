use super::cvars::CVarRegistry;
use crate::hud::PlayerStats;
use bevy::prelude::*;

/// Handle the savecvars command - saves all console variables to data/cvars.yaml
pub fn cmd_savecvars(
    _tokens: &[&str],
    _stats: &mut ResMut<PlayerStats>,
    cvars: &mut ResMut<CVarRegistry>,
) -> String {
    match cvars.save_to_yaml("data/cvars.yaml") {
        Ok(_) => "CVars saved to data/cvars.yaml".to_string(),
        Err(e) => format!("Failed to save cvars: {}", e),
    }
}
