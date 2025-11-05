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

/// Component attached to actor entities in the game world
#[derive(Component, Debug)]
pub struct Actor {
    pub actor_type: String,
    pub health: f32,
    pub max_health: f32,
    pub scale: f32,
}

/// Position data for actors in the map file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorPosition {
    pub x: f32,
    pub y: f32,
    pub actor_type: String,
}
