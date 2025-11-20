use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    pub fn from_positions(positions: &[Vec3]) -> Self {
        if positions.is_empty() {
            return Self::empty();
        }

        let mut min = positions[0];
        let mut max = positions[0];

        for pos in positions.iter().skip(1) {
            min = min.min(*pos);
            max = max.max(*pos);
        }

        Self { min, max }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    pub fn merge(&self, other: &BBox) -> BBox {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }
        BBox {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn sphere_radius(&self) -> f32 {
        if self.is_empty() {
            return 0.0;
        }
        (self.max - self.min).length() * 0.5
    }
}
