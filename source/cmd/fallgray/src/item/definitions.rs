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
