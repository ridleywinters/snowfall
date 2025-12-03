use super::cvars::CVarRegistry;
use crate::hud::PlayerStats;
use bevy::prelude::*;

/// Handle the getvar command - retrieves a console variable value
pub fn cmd_getvar(
    tokens: &[&str],
    _stats: &mut ResMut<PlayerStats>,
    cvars: &mut ResMut<CVarRegistry>,
) -> String {
    if tokens.len() < 2 {
        return "usage: getvar <variable>".to_string();
    }

    let var_name = tokens[1];

    match cvars.get(var_name) {
        Some(value) => format!("{}", value),
        None => format!("Variable not found: {}", var_name),
    }
}
