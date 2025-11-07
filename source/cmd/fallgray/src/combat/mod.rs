pub mod actor_attack;
/// Combat system module
///
/// Handles weapon attacks, damage calculation, and combat states.
/// Organized into submodules for clarity and maintainability.
pub mod attack_state;
pub mod audio_feedback;
pub mod damage;
pub mod status_effects;
pub mod visual_feedback;
pub mod weapon;

pub use actor_attack::{
    handle_actor_hit, update_actor_attack_animation, update_actor_attacks, update_actor_stun,
};
pub use attack_state::{AttackState, CombatInput, StateTransition};
pub use audio_feedback::{CombatAudio, play_hit_sound, play_swing_sound};
pub use damage::calculate_damage;
pub use status_effects::{apply_status_effect, update_status_effects};
pub use visual_feedback::{
    CameraShake, spawn_blood_particles, spawn_damage_number, update_blood_particles,
    update_camera_shake, update_damage_numbers,
};
pub use weapon::WeaponDefinitions;
