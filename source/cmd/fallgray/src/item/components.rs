use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Item {
    pub interaction_radius: f32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ItemPosition {
    pub x: f32,
    pub y: f32,
    #[serde(default = "default_item_type")]
    pub item_type: String,
}

pub fn default_item_type() -> String {
    "apple".to_string()
}
