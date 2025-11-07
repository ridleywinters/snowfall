/// Visual feedback for combat actions
///
/// Handles blood particles, damage numbers, and other visual effects.
use bevy::prelude::*;

/// Component for damage number floating text
#[derive(Component, Debug)]
pub struct DamageNumber {
    /// Time remaining before despawn
    pub lifetime: f32,

    /// Initial spawn time for animation
    pub spawn_time: f32,
}

impl DamageNumber {
    pub fn new() -> Self {
        Self {
            lifetime: 1.5,
            spawn_time: 1.5,
        }
    }
}

/// System to update damage number positions and lifetime
pub fn update_damage_numbers(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber, &mut TextColor)>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut damage_num, mut text_color) in query.iter_mut() {
        // Move upward
        transform.translation.z += time.delta_secs() * 2.0;

        // Fade out based on remaining lifetime
        let alpha = (damage_num.lifetime / damage_num.spawn_time).clamp(0.0, 1.0);
        text_color.0 = text_color.0.with_alpha(alpha);

        // Update lifetime
        damage_num.lifetime -= time.delta_secs();

        // Despawn when expired
        if damage_num.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn a damage number at the given world position
pub fn spawn_damage_number(
    commands: &mut Commands,
    _asset_server: &Res<AssetServer>,
    position: Vec3,
    damage: i32,
    critical: bool,
) {
    let color = if critical {
        Color::srgb(1.0, 0.8, 0.2) // Gold for crits
    } else {
        Color::srgb(1.0, 0.2, 0.2) // Red for normal hits
    };

    let font_size = if critical { 48.0 } else { 32.0 };

    commands.spawn((
        Text2d::new(damage.to_string()),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position + Vec3::new(0.0, 0.0, 2.0)),
        DamageNumber::new(),
    ));
}

/// Component for blood particle effect
#[derive(Component, Debug)]
pub struct BloodParticle {
    /// Velocity of particle
    pub velocity: Vec3,

    /// Time remaining before despawn
    pub lifetime: f32,
}

impl BloodParticle {
    pub fn new(velocity: Vec3) -> Self {
        Self {
            velocity,
            lifetime: 0.5,
        }
    }
}

/// System to update blood particles
pub fn update_blood_particles(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut BloodParticle)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle) in query.iter_mut() {
        // Apply velocity
        transform.translation += particle.velocity * dt;

        // Apply gravity (Z- since Z+ is up)
        particle.velocity.z -= 9.8 * dt;

        // Update lifetime
        particle.lifetime -= dt;

        // Despawn when expired
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn blood particles at the given position
pub fn spawn_blood_particles(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    count: u32,
) {
    let blood_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.0, 0.0),
        unlit: true,
        ..default()
    });

    let particle_mesh = meshes.add(Sphere::new(0.05));

    for _ in 0..count {
        // Random velocity
        let velocity = Vec3::new(
            (rand::random::<f32>() - 0.5) * 4.0,
            (rand::random::<f32>() - 0.5) * 4.0,
            rand::random::<f32>() * 3.0,
        );

        commands.spawn((
            Mesh3d(particle_mesh.clone()),
            MeshMaterial3d(blood_material.clone()),
            Transform::from_translation(position),
            BloodParticle::new(velocity),
        ));
    }
}
