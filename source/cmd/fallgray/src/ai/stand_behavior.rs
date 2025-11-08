use super::ActorBehavior;
use crate::map::Map;
use bevy::prelude::*;

/// Stand behavior - actor does nothing
pub struct StandBehavior;

impl ActorBehavior for StandBehavior {
    fn update(
        &mut self,
        _transform: &mut Transform,
        _map: &Map,
        _delta_time: f32,
        _speed_multiplier: f32,
        _player_position: Option<Vec2>,
        _actor: &crate::ai::ActorData,
    ) -> bool {
        false // Not moving
    }

    fn get_label(&self) -> &str {
        "stand"
    }
}
