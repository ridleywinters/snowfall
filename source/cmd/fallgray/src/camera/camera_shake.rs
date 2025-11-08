/// Camera shake effect system
///
/// Provides camera shake visual feedback for combat actions and other events.
use bevy::prelude::*;

/// Component for camera shake effect
#[derive(Component, Debug)]
pub struct CameraShake {
    /// Intensity of shake (displacement magnitude)
    pub intensity: f32,

    /// Duration remaining in seconds
    pub duration: f32,

    /// Frequency of shake oscillation
    pub frequency: f32,

    /// Base camera position when shake started
    pub base_position: Vec3,
}

impl CameraShake {
    /// Create a new camera shake effect
    pub fn new(intensity: f32, duration: f32, base_position: Vec3) -> Self {
        Self {
            intensity,
            duration,
            frequency: 20.0, // Default shake frequency
            base_position,
        }
    }

    /// Create shake for a hit effect (base_position will be set when inserted)
    pub fn hit_shake() -> Self {
        Self::new(0.1, 0.15, Vec3::ZERO)
    }

    /// Create shake for a critical hit (base_position will be set when inserted)
    pub fn critical_shake() -> Self {
        Self::new(0.2, 0.25, Vec3::ZERO)
    }
}

/// System to update camera shake
pub fn update_camera_shake(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut CameraShake)>,
    mut commands: Commands,
) {
    let mut to_remove = Vec::new();

    for (entity, mut transform, mut shake) in query.iter_mut() {
        // Store base position on first frame (when base_position is zero)
        if shake.base_position == Vec3::ZERO {
            shake.base_position = transform.translation;
        }

        if shake.duration <= 0.0 {
            // Reset to base position before removing
            transform.translation = shake.base_position;
            to_remove.push(entity);
            continue;
        }

        // Calculate shake offset using sine wave
        let elapsed = time.elapsed_secs();
        let shake_x = (elapsed * shake.frequency).sin() * shake.intensity;
        let shake_y = (elapsed * shake.frequency * 1.3).cos() * shake.intensity * 0.7;
        let shake_z = (elapsed * shake.frequency * 0.8).sin() * shake.intensity * 0.5;

        // Update
        transform.translation = shake.base_position + Vec3::new(shake_x, shake_y, shake_z);
        shake.duration -= time.delta_secs();
    }

    // Clean up expired shakes
    for entity in to_remove {
        commands.entity(entity).remove::<CameraShake>();
    }
}
