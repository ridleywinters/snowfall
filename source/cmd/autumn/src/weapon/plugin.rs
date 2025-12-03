use bevy::prelude::*;
use crate::game_state::GameState;
use super::systems::{update_weapon_swing, update_weapon_swing_collision};

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_weapon_swing,
                update_weapon_swing_collision,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
