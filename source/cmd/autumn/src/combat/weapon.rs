/// Weapon definition system
/// 
/// Loads weapon stats and animation keyframes from YAML files.
/// Dynamically registers CVars for each weapon to allow runtime tuning.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::damage::DamageType;
use crate::scripting::{CVarRegistry, CVarValue};

/// Animation keyframe positions and rotations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimationKeyframe {
    /// Position offset (X, Y, Z)
    pub position: Vec3,
    
    /// Rotation (Z-axis rotation, Y-axis rotation)
    pub rotation: (f32, f32),
}

/// Complete definition of a weapon type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeaponDefinition {
    /// Unique weapon type identifier (e.g., "sword", "axe")
    pub weapon_type: String,
    
    /// Base attack damage
    pub attack_power: i32,
    
    /// Total duration of attack animation in seconds
    pub swing_duration: f32,
    
    /// Maximum time to charge attack in seconds
    pub max_charge_time: f32,
    
    /// Damage multiplier bonus when fully charged (e.g., 0.5 = +50%)
    pub charge_bonus: f32,
    
    /// Attack range in world units
    pub range: f32,
    
    /// Width of attack hitbox (lateral spread)
    pub hitbox_width: f32,
    
    /// Height of attack hitbox (vertical reach)
    pub hitbox_height: f32,
    
    /// Type of damage this weapon deals
    pub damage_type: DamageType,
    
    /// Animation keyframes for different attack phases
    pub rest_keyframe: AnimationKeyframe,
    pub windup_keyframe: AnimationKeyframe,
    pub swing_keyframe: AnimationKeyframe,
    pub thrust_keyframe: AnimationKeyframe,
}

/// Resource holding all loaded weapon definitions
#[derive(Resource, Default)]
pub struct WeaponDefinitions {
    pub weapons: HashMap<String, WeaponDefinition>,
}

impl WeaponDefinitions {
    /// Load weapon definitions from YAML file
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let yaml = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;
        
        let weapons: HashMap<String, WeaponDefinition> = serde_yaml::from_str(&yaml)
            .map_err(|e| format!("Failed to parse {}: {}", path, e))?;
        
        Ok(Self { weapons })
    }
    
    /// Register CVars for all loaded weapons
    /// 
    /// Creates runtime-tunable variables like:
    /// - weapon.sword.attack_power
    /// - weapon.sword.swing_duration
    /// - weapon.sword.windup_pos_x
    /// etc.
    pub fn register_cvars(&self, cvars: &mut CVarRegistry) {
        for (weapon_type, weapon) in &self.weapons {
            let prefix = format!("weapon.{}", weapon_type);
            
            // Register stat CVars
            let _ = cvars.init(&format!("{}.attack_power", prefix), CVarValue::Int32(weapon.attack_power));
            let _ = cvars.init(&format!("{}.swing_duration", prefix), CVarValue::F32(weapon.swing_duration));
            let _ = cvars.init(&format!("{}.max_charge_time", prefix), CVarValue::F32(weapon.max_charge_time));
            let _ = cvars.init(&format!("{}.charge_bonus", prefix), CVarValue::F32(weapon.charge_bonus));
            let _ = cvars.init(&format!("{}.range", prefix), CVarValue::F32(weapon.range));
            let _ = cvars.init(&format!("{}.hitbox_width", prefix), CVarValue::F32(weapon.hitbox_width));
            let _ = cvars.init(&format!("{}.hitbox_height", prefix), CVarValue::F32(weapon.hitbox_height));
            
            // Register animation keyframe CVars - Rest
            let _ = cvars.init(&format!("{}.rest_pos_x", prefix), CVarValue::F32(weapon.rest_keyframe.position.x));
            let _ = cvars.init(&format!("{}.rest_pos_y", prefix), CVarValue::F32(weapon.rest_keyframe.position.y));
            let _ = cvars.init(&format!("{}.rest_pos_z", prefix), CVarValue::F32(weapon.rest_keyframe.position.z));
            let _ = cvars.init(&format!("{}.rest_rotation_z", prefix), CVarValue::F32(weapon.rest_keyframe.rotation.0));
            let _ = cvars.init(&format!("{}.rest_rotation_y", prefix), CVarValue::F32(weapon.rest_keyframe.rotation.1));
            
            // Register animation keyframe CVars - Windup
            let _ = cvars.init(&format!("{}.windup_pos_x", prefix), CVarValue::F32(weapon.windup_keyframe.position.x));
            let _ = cvars.init(&format!("{}.windup_pos_y", prefix), CVarValue::F32(weapon.windup_keyframe.position.y));
            let _ = cvars.init(&format!("{}.windup_pos_z", prefix), CVarValue::F32(weapon.windup_keyframe.position.z));
            let _ = cvars.init(&format!("{}.windup_rotation_z", prefix), CVarValue::F32(weapon.windup_keyframe.rotation.0));
            let _ = cvars.init(&format!("{}.windup_rotation_y", prefix), CVarValue::F32(weapon.windup_keyframe.rotation.1));
            
            // Register animation keyframe CVars - Swing
            let _ = cvars.init(&format!("{}.swing_pos_x", prefix), CVarValue::F32(weapon.swing_keyframe.position.x));
            let _ = cvars.init(&format!("{}.swing_pos_y", prefix), CVarValue::F32(weapon.swing_keyframe.position.y));
            let _ = cvars.init(&format!("{}.swing_pos_z", prefix), CVarValue::F32(weapon.swing_keyframe.position.z));
            let _ = cvars.init(&format!("{}.swing_rotation_z", prefix), CVarValue::F32(weapon.swing_keyframe.rotation.0));
            let _ = cvars.init(&format!("{}.swing_rotation_y", prefix), CVarValue::F32(weapon.swing_keyframe.rotation.1));
            
            // Register animation keyframe CVars - Thrust
            let _ = cvars.init(&format!("{}.thrust_pos_x", prefix), CVarValue::F32(weapon.thrust_keyframe.position.x));
            let _ = cvars.init(&format!("{}.thrust_pos_y", prefix), CVarValue::F32(weapon.thrust_keyframe.position.y));
            let _ = cvars.init(&format!("{}.thrust_pos_z", prefix), CVarValue::F32(weapon.thrust_keyframe.position.z));
            let _ = cvars.init(&format!("{}.thrust_rotation_z", prefix), CVarValue::F32(weapon.thrust_keyframe.rotation.0));
            let _ = cvars.init(&format!("{}.thrust_rotation_y", prefix), CVarValue::F32(weapon.thrust_keyframe.rotation.1));
        }
    }
    
    /// Get a weapon definition with values updated from CVars
    /// 
    /// This allows runtime modifications via the console to take effect
    pub fn get_with_cvars(&self, weapon_type: &str, cvars: &CVarRegistry) -> Option<WeaponDefinition> {
        let mut weapon = self.weapons.get(weapon_type)?.clone();
        let prefix = format!("weapon.{}", weapon_type);
        
        // Update stats from CVars if they exist
        if cvars.exists(&format!("{}.attack_power", prefix)) {
            weapon.attack_power = cvars.get_i32(&format!("{}.attack_power", prefix));
        }
        if cvars.exists(&format!("{}.swing_duration", prefix)) {
            weapon.swing_duration = cvars.get_f32(&format!("{}.swing_duration", prefix));
        }
        if cvars.exists(&format!("{}.max_charge_time", prefix)) {
            weapon.max_charge_time = cvars.get_f32(&format!("{}.max_charge_time", prefix));
        }
        if cvars.exists(&format!("{}.charge_bonus", prefix)) {
            weapon.charge_bonus = cvars.get_f32(&format!("{}.charge_bonus", prefix));
        }
        if cvars.exists(&format!("{}.range", prefix)) {
            weapon.range = cvars.get_f32(&format!("{}.range", prefix));
        }
        if cvars.exists(&format!("{}.hitbox_width", prefix)) {
            weapon.hitbox_width = cvars.get_f32(&format!("{}.hitbox_width", prefix));
        }
        if cvars.exists(&format!("{}.hitbox_height", prefix)) {
            weapon.hitbox_height = cvars.get_f32(&format!("{}.hitbox_height", prefix));
        }
        
        // Update rest keyframe from CVars
        weapon.rest_keyframe.position.x = cvars.get_f32(&format!("{}.rest_pos_x", prefix));
        weapon.rest_keyframe.position.y = cvars.get_f32(&format!("{}.rest_pos_y", prefix));
        weapon.rest_keyframe.position.z = cvars.get_f32(&format!("{}.rest_pos_z", prefix));
        weapon.rest_keyframe.rotation.0 = cvars.get_f32(&format!("{}.rest_rotation_z", prefix));
        weapon.rest_keyframe.rotation.1 = cvars.get_f32(&format!("{}.rest_rotation_y", prefix));
        
        // Update windup keyframe from CVars
        weapon.windup_keyframe.position.x = cvars.get_f32(&format!("{}.windup_pos_x", prefix));
        weapon.windup_keyframe.position.y = cvars.get_f32(&format!("{}.windup_pos_y", prefix));
        weapon.windup_keyframe.position.z = cvars.get_f32(&format!("{}.windup_pos_z", prefix));
        weapon.windup_keyframe.rotation.0 = cvars.get_f32(&format!("{}.windup_rotation_z", prefix));
        weapon.windup_keyframe.rotation.1 = cvars.get_f32(&format!("{}.windup_rotation_y", prefix));
        
        // Update swing keyframe from CVars
        weapon.swing_keyframe.position.x = cvars.get_f32(&format!("{}.swing_pos_x", prefix));
        weapon.swing_keyframe.position.y = cvars.get_f32(&format!("{}.swing_pos_y", prefix));
        weapon.swing_keyframe.position.z = cvars.get_f32(&format!("{}.swing_pos_z", prefix));
        weapon.swing_keyframe.rotation.0 = cvars.get_f32(&format!("{}.swing_rotation_z", prefix));
        weapon.swing_keyframe.rotation.1 = cvars.get_f32(&format!("{}.swing_rotation_y", prefix));
        
        // Update thrust keyframe from CVars
        weapon.thrust_keyframe.position.x = cvars.get_f32(&format!("{}.thrust_pos_x", prefix));
        weapon.thrust_keyframe.position.y = cvars.get_f32(&format!("{}.thrust_pos_y", prefix));
        weapon.thrust_keyframe.position.z = cvars.get_f32(&format!("{}.thrust_pos_z", prefix));
        weapon.thrust_keyframe.rotation.0 = cvars.get_f32(&format!("{}.thrust_rotation_z", prefix));
        weapon.thrust_keyframe.rotation.1 = cvars.get_f32(&format!("{}.thrust_rotation_y", prefix));
        
        Some(weapon)
    }
}
