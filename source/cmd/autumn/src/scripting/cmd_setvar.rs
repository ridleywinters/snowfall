use super::cvars::{CVarRegistry, CVarValue};
use crate::hud::PlayerStats;
use bevy::prelude::*;

/// Parse a string value into the appropriate CVarValue based on the existing variable's type
pub fn parse_value_for_type(
    value_str: &str,
    existing_var: &CVarValue,
) -> Result<CVarValue, String> {
    match existing_var {
        CVarValue::F32(_) => match value_str.parse::<f32>() {
            Ok(v) => Ok(CVarValue::F32(v)),
            Err(_) => Err(format!("Invalid f32 value: {}", value_str)),
        },
        CVarValue::Int32(_) => match value_str.parse::<i32>() {
            Ok(v) => Ok(CVarValue::Int32(v)),
            Err(_) => Err(format!("Invalid int32 value: {}", value_str)),
        },
        CVarValue::Bool(_) => match value_str {
            "true" => Ok(CVarValue::Bool(true)),
            "false" => Ok(CVarValue::Bool(false)),
            _ => Err(format!(
                "Invalid bool value: {} (use true or false)",
                value_str
            )),
        },
        CVarValue::String(_) => Ok(CVarValue::String(value_str.to_string())),
    }
}

/// Worker function that handles setvar logic without Bevy dependencies
pub fn cmd_setvar_worker(tokens: &[&str], cvars: &mut CVarRegistry) -> String {
    if tokens.len() < 3 {
        return "usage: setvar <variable> <value>".to_string();
    }

    let var_name = tokens[1];
    let value_str = tokens[2];

    // Look up the existing variable to determine its type
    let existing_var = match cvars.get(var_name) {
        Some(v) => v.clone(),
        None => return format!("Variable '{}' does not exist", var_name),
    };

    // Convert the string input to the appropriate type based on existing variable type
    let new_value = match parse_value_for_type(value_str, &existing_var) {
        Ok(v) => v,
        Err(e) => return e,
    };

    match cvars.set(var_name, new_value) {
        Ok(_) => format!("{} = {}", var_name, value_str),
        Err(e) => e,
    }
}

/// Handle the setvar command - sets a console variable value (Bevy wrapper)
pub fn cmd_setvar(
    tokens: &[&str],
    _stats: &mut ResMut<PlayerStats>,
    cvars: &mut ResMut<CVarRegistry>,
) -> String {
    cmd_setvar_worker(tokens, cvars)
}
