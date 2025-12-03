use super::cvars::CVarRegistry;
use crate::hud::PlayerStats;
use bevy::prelude::*;

pub fn cmd_add_gold(
    tokens: &[&str],
    stats: &mut ResMut<PlayerStats>,
    _cvars: &mut ResMut<CVarRegistry>,
) -> String {
    if tokens.len() < 2 {
        return "usage: add_gold <amount>".to_string();
    }

    let Ok(amount) = tokens[1].parse::<i32>() else {
        return format!("Invalid gold amount: {}", tokens[1]);
    };

    stats.gold += amount;
    format!("Added {} gold, new value: {}", amount, stats.gold)
}
