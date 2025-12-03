/// Damage calculation system
///
/// Handles computing damage values based on weapon stats, charge level,
/// critical hits, and target resistances.
use super::weapon::WeaponDefinition;
use serde::{Deserialize, Serialize};

/// Types of damage that can be dealt
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
}

/// Result of a damage calculation
#[derive(Clone, Debug)]
pub struct DamageResult {
    /// Final damage amount to apply
    pub amount: i32,

    /// Whether this was a critical hit
    pub critical: bool,

    /// Type of damage dealt
    pub damage_type: DamageType,
}

/// Calculate damage for a weapon attack
///
/// Takes into account:
/// - Base weapon attack power
/// - Charge ratio (0.0 to 1.0)
/// - Critical hit chance (5%)
/// - Target armor
/// - Target resistance to damage type
pub fn calculate_damage(
    weapon: &WeaponDefinition,
    charge_ratio: f32,
    target_armor: i32,
    target_resistance: f32,
) -> DamageResult {
    // Start with base weapon damage
    let mut damage = weapon.attack_power as f32;

    // Apply charge bonus
    damage *= 1.0 + (charge_ratio * weapon.charge_bonus);

    // Roll for critical hit (5% chance)
    let critical = rand::random::<f32>() < 0.05;
    if critical {
        damage *= 2.0;
    }

    // Apply armor reduction (flat subtraction)
    damage -= target_armor as f32;

    // Apply resistance multiplier (0.0 = no damage, 1.0 = full damage)
    damage *= 1.0 - target_resistance.clamp(0.0, 1.0);

    // Ensure damage is at least 0
    let final_damage = damage.max(0.0).round() as i32;

    DamageResult {
        amount: final_damage,
        critical,
        damage_type: weapon.damage_type,
    }
}
