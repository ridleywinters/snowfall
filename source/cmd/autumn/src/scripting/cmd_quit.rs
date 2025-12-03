use super::cvars::CVarRegistry;
use crate::hud::PlayerStats;
use bevy::prelude::*;

pub fn cmd_quit(
    _tokens: &[&str],
    _stats: &mut ResMut<PlayerStats>,
    _cvars: &mut ResMut<CVarRegistry>,
) -> String {
    println!("Exiting...");
    std::process::exit(0);
}
