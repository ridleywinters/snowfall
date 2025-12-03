use bevy::prelude::*;
use crate::game_state::GameState;
use super::systems::{update_actor_death, update_actor_health_indicators};

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_actor_death,
                update_actor_health_indicators,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
