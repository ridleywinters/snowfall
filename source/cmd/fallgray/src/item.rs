use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct ItemDefinition {
    pub image: String,
    pub script: String,
    pub scale: f32,
    pub effects: Vec<String>,
}

#[derive(Deserialize)]
pub struct ItemDefinitionsFile {
    pub items: HashMap<String, ItemDefinition>,
}

#[derive(Resource)]
pub struct ItemDefinitions {
    pub items: HashMap<String, ItemDefinition>,
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

#[derive(Component)]
pub struct Item {
    pub interaction_radius: f32,
}
