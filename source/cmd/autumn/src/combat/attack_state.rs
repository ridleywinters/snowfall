/// Attack state machine for weapon combat
/// 
/// Manages the progression through attack phases: idle → windup → swing → thrust → recovery
/// Each state tracks its own progress and determines when to transition to the next state.

use super::weapon::WeaponDefinition;

/// Represents the current state of a weapon attack
#[derive(Clone, Debug, PartialEq)]
pub enum AttackState {
    /// Weapon at rest, ready to attack
    Idle,
    
    /// Pulling weapon back in preparation
    /// Progress: 0.0 (start) to 1.0 (ready to swing)
    Windup { progress: f32 },
    
    /// Horizontal or arc swing motion
    /// Progress: 0.0 (start) to 1.0 (ready to thrust)
    /// hit_active: true when hitbox should detect collisions
    Swing { progress: f32, hit_active: bool },
    
    /// Forward thrust or final strike
    /// Progress: 0.0 (start) to 1.0 (complete)
    /// hit_active: true when hitbox should detect collisions
    Thrust { progress: f32, hit_active: bool },
    
    /// Returning to rest position
    /// Progress: 0.0 (start) to 1.0 (back to idle)
    Recovery { progress: f32 },
}

/// Result of updating an attack state
#[derive(Clone, Debug, PartialEq)]
pub enum StateTransition {
    /// Stay in current state
    Stay,
    
    /// Transition to a new state
    To(AttackState),
    
    /// Trigger hit detection this frame
    TriggerHitDetection,
}

/// Input state for combat actions
#[derive(Clone, Debug, Default)]
pub struct CombatInput {
    /// Attack button was just pressed this frame
    pub attack_pressed: bool,
    
    /// Attack button is currently held down
    pub attack_held: bool,
}

impl AttackState {
    /// Update the attack state based on elapsed time and input
    /// 
    /// Returns a StateTransition indicating what should happen next.
    /// The weapon definition determines phase durations and when hits are active.
    pub fn update(
        &mut self,
        dt: f32,
        input: &CombatInput,
        weapon: &WeaponDefinition,
    ) -> StateTransition {
        // Calculate normalized phase boundaries (0.0 to 1.0)
        let windup_end = 0.15;
        let swing_end = 0.50;
        let thrust_end = 0.80;
        // recovery_end = 1.0
        
        match self {
            AttackState::Idle => {
                // Transition to windup when attack is pressed
                if input.attack_pressed {
                    StateTransition::To(AttackState::Windup { progress: 0.0 })
                } else {
                    StateTransition::Stay
                }
            }
            
            AttackState::Windup { progress } => {
                // Advance through windup phase
                let phase_duration = weapon.swing_duration * windup_end;
                *progress += dt / phase_duration;
                
                if *progress >= 1.0 {
                    // Windup complete, start swing
                    StateTransition::To(AttackState::Swing {
                        progress: 0.0,
                        hit_active: false,
                    })
                } else {
                    StateTransition::Stay
                }
            }
            
            AttackState::Swing { progress, hit_active } => {
                // Advance through swing phase
                let phase_duration = weapon.swing_duration * (swing_end - windup_end);
                *progress += dt / phase_duration;
                
                // Activate hitbox at 50% through swing phase
                if *progress >= 0.5 && !*hit_active {
                    *hit_active = true;
                    return StateTransition::TriggerHitDetection;
                }
                
                if *progress >= 1.0 {
                    // Swing complete, start thrust
                    StateTransition::To(AttackState::Thrust {
                        progress: 0.0,
                        hit_active: false,
                    })
                } else {
                    StateTransition::Stay
                }
            }
            
            AttackState::Thrust { progress, hit_active } => {
                // Advance through thrust phase
                let phase_duration = weapon.swing_duration * (thrust_end - swing_end);
                *progress += dt / phase_duration;
                
                // Activate hitbox at 50% through thrust phase
                if *progress >= 0.5 && !*hit_active {
                    *hit_active = true;
                    return StateTransition::TriggerHitDetection;
                }
                
                if *progress >= 1.0 {
                    // Thrust complete, start recovery
                    StateTransition::To(AttackState::Recovery { progress: 0.0 })
                } else {
                    StateTransition::Stay
                }
            }
            
            AttackState::Recovery { progress } => {
                // Advance through recovery phase
                let phase_duration = weapon.swing_duration * (1.0 - thrust_end);
                *progress += dt / phase_duration;
                
                if *progress >= 1.0 {
                    // Recovery complete, return to idle
                    StateTransition::To(AttackState::Idle)
                } else {
                    StateTransition::Stay
                }
            }
        }
    }
    
    /// Check if hit detection should be active in this state
    pub fn is_hit_active(&self) -> bool {
        match self {
            AttackState::Swing { hit_active, .. } => *hit_active,
            AttackState::Thrust { hit_active, .. } => *hit_active,
            _ => false,
        }
    }
    
    /// Get the current progress through the entire attack (0.0 to 1.0)
    /// Used for animation interpolation
    pub fn get_overall_progress(&self) -> f32 {
        let windup_end = 0.15;
        let swing_end = 0.50;
        let thrust_end = 0.80;
        
        match self {
            AttackState::Idle => 0.0,
            AttackState::Windup { progress } => progress * windup_end,
            AttackState::Swing { progress, .. } => {
                windup_end + progress * (swing_end - windup_end)
            }
            AttackState::Thrust { progress, .. } => {
                swing_end + progress * (thrust_end - swing_end)
            }
            AttackState::Recovery { progress } => {
                thrust_end + progress * (1.0 - thrust_end)
            }
        }
    }
}
