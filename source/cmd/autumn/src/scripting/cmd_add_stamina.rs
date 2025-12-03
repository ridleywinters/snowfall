use super::cvars::CVarRegistry;
use crate::hud::PlayerStats;
use bevy::prelude::*;

pub fn cmd_add_stamina(
    tokens: &[&str],
    stats: &mut ResMut<PlayerStats>,
    _cvars: &mut ResMut<CVarRegistry>,
) -> String {
    if tokens.len() < 2 {
        return "usage: add_stamina <amount>".to_string();
    }

    let Ok(amount) = tokens[1].parse::<f32>() else {
        return format!("Invalid stamina amount: {}", tokens[1]);
    };

    stats.stamina = (stats.stamina + amount).min(100.0);
    format!("Added {} stamina, new value: {}", amount, stats.stamina)
}
