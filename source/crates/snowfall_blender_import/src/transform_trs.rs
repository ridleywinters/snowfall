use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct TransformTRS {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}
