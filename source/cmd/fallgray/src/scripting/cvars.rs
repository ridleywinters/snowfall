/// Console Variables (CVars) management
///
/// Provides a registry for defining, setting, and getting console variables
/// of different types (float, int, string).  The purpose of console variables
/// is to allow for runtime modifications of game settings and parameters
/// either manually via the console or programmatically via scripts.
///
/// The idea is borrowed from old Quake-style console variables.
///
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a console variable value
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum CVarValue {
    Float(f32),
    Int(i32),
    String(String),
}

impl CVarValue {
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            CVarValue::Float(v) => Some(*v),
            CVarValue::Int(v) => Some(*v as f32),
            CVarValue::String(s) => s.parse().ok(),
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            CVarValue::Float(v) => Some(*v as i32),
            CVarValue::Int(v) => Some(*v),
            CVarValue::String(s) => s.parse().ok(),
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            CVarValue::Float(v) => v.to_string(),
            CVarValue::Int(v) => v.to_string(),
            CVarValue::String(s) => s.clone(),
        }
    }
}

impl std::fmt::Display for CVarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CVarValue::Float(v) => write!(f, "{}", v),
            CVarValue::Int(v) => write!(f, "{}", v),
            CVarValue::String(s) => write!(f, "{}", s),
        }
    }
}

/// Resource that stores all console variables
#[derive(Resource, Default)]
pub struct CVarRegistry {
    vars: HashMap<String, CVarValue>,
}

impl CVarRegistry {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    /// Check if a variable name is valid
    pub fn is_valid_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let mut chars = name.chars();

        // First character must be A-Z, a-z, or _
        if let Some(first) = chars.next() {
            if !first.is_ascii_alphabetic() {
                return false;
            }
        } else {
            return false;
        }

        // Remaining characters must be A-Z, a-z, 0-9, _, or .
        for ch in chars {
            if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.' {
                return false;
            }
        }

        true
    }

    pub fn init(&mut self, name: &str, value: CVarValue) -> Result<(), String> {
        if !Self::is_valid_name(name) {
            return Err(format!("Invalid variable name: {}", name));
        }
        if self.vars.contains_key(name) {
            return Err(format!("Variable already exists: {}", name));
        }

        self.vars.insert(name.to_string(), value);
        Ok(())
    }

    pub fn init_f32(&mut self, name: &str, value: f32) {
        self.init(name, CVarValue::Float(value)).unwrap();
    }

    pub fn set(&mut self, name: &str, value: CVarValue) -> Result<(), String> {
        let existing = self
            .vars
            .get(name)
            .ok_or_else(|| format!("Variable does not exist: {}", name))?;

        // Check that the new value type matches the existing type
        match (existing, &value) {
            (CVarValue::Float(_), CVarValue::Float(_)) => {}
            (CVarValue::Int(_), CVarValue::Int(_)) => {}
            (CVarValue::String(_), CVarValue::String(_)) => {}
            _ => {
                return Err(format!(
                    "Type mismatch for variable '{}': cannot change from {:?} to {:?}",
                    name,
                    std::mem::discriminant(existing),
                    std::mem::discriminant(&value)
                ));
            }
        }

        self.vars.insert(name.to_string(), value);
        Ok(())
    }

    pub fn set_f32(&mut self, name: &str, value: f32) {
        self.set(name, CVarValue::Float(value)).unwrap();
    }

    pub fn get(&self, name: &str) -> Option<&CVarValue> {
        self.vars.get(name)
    }

    pub fn get_f32(&self, name: &str) -> f32 {
        self.vars.get(name).and_then(|v| v.as_f32()).unwrap()
    }

    pub fn get_i32(&self, name: &str) -> i32 {
        self.vars.get(name).and_then(|v| v.as_i32()).unwrap()
    }

    pub fn get_string(&self, name: &str) -> String {
        self.vars.get(name).map(|v| v.as_string()).unwrap()
    }

    pub fn exists(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    pub fn list(&self) -> Vec<(String, CVarValue)> {
        let mut result: Vec<(String, CVarValue)> = self
            .vars
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Sort alphabetically by variable name
        result.sort_by(|a, b| a.0.cmp(&b.0));

        result
    }

    pub fn save_to_yaml(&self, path: &str) -> Result<(), String> {
        // Sort variables alphabetically by name and create a YAML mapping
        let mut sorted_vars: Vec<(&String, &CVarValue)> = self.vars.iter().collect();
        sorted_vars.sort_by(|a, b| a.0.cmp(b.0));
        
        // Create a YAML mapping that preserves insertion order
        let mut mapping = serde_yaml::Mapping::new();
        for (key, value) in sorted_vars {
            let key_value = serde_yaml::Value::String(key.clone());
            let value_value = serde_yaml::to_value(value)
                .map_err(|e| format!("Failed to serialize value for {}: {}", key, e))?;
            mapping.insert(key_value, value_value);
        }
        
        let yaml = serde_yaml::to_string(&mapping)
            .map_err(|e| format!("Failed to serialize cvars: {}", e))?;

        std::fs::write(path, yaml).map_err(|e| format!("Failed to write cvars.yaml: {}", e))?;

        Ok(())
    }
}
