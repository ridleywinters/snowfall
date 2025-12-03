use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Definition of an actor type loaded from YAML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorDefinition {
    pub sprite: String,
    pub scale: f32,
    pub max_health: f32,
    pub on_hit: String,
    pub on_death: String,
    #[serde(default = "default_behavior")]
    pub behavior: String,
    #[serde(default = "default_speed")]
    pub speed: f32,
    #[serde(default)]
    pub attack_damage: i32,
    #[serde(default = "default_attack_range")]
    pub attack_range: f32,
    #[serde(default = "default_attack_cooldown")]
    pub attack_cooldown: f32,
}

fn default_behavior() -> String {
    "wander".to_string()
}

fn default_speed() -> f32 {
    1.0
}

fn default_attack_range() -> f32 {
    4.0
}

fn default_attack_cooldown() -> f32 {
    1.2
}

/// File structure for loading actor definitions from YAML
#[derive(Debug, Deserialize, Serialize)]
pub struct ActorDefinitionsFile {
    pub actors: HashMap<String, ActorDefinition>,
}

/// Resource containing all actor definitions
#[derive(Resource, Debug)]
pub struct ActorDefinitions {
    pub actors: HashMap<String, ActorDefinition>,
}
