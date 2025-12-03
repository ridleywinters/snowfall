use super::cvars::CVarRegistry;
use crate::hud::PlayerStats;
use bevy::prelude::*;

/// Handle the listvars command - lists all console variables
pub fn cmd_listvars(
    _tokens: &[&str],
    _stats: &mut ResMut<PlayerStats>,
    cvars: &mut ResMut<CVarRegistry>,
) -> String {
    let vars = cvars.list();

    if vars.is_empty() {
        return "No variables defined".to_string();
    }

    let mut output = format!("{} variables:", vars.len());
    for (name, value) in vars {
        output.push_str(&format!("\n  {} = {}", name, value));
    }
    output
}
