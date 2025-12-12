use glam::Vec3;

/// An axis-aligned bounding box.
///
/// Initialized to invalid infinity values (min > max) to represent an empty box.
/// Designed with THREE.Box3 from three.js in mind.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BBox {
    //=========================================================================
    // Construction
    //=========================================================================

    /// Creates a new BBox with invalid infinity values (empty box).
    pub fn new() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    /// Creates a BBox from min and max points.
    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Creates a BBox from a center point and size.
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Creates a BBox that contains all given points.
    pub fn from_points(points: &[Vec3]) -> Self {
        let mut bbox = Self::new();
        bbox.expand_by_points(points);
        bbox
    }

    /// Creates a BBox from an array of positions by computing min/max.
    pub fn from_array(position_array: &[Vec3]) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for pos in position_array.iter() {
            min = min.min(*pos);
            max = max.max(*pos);
        }
        Self::from_min_max(min, max)
    }

    //=========================================================================
    // Properties
    //=========================================================================

    /// Returns true if the bounding box is empty (invalid).
    pub fn is_empty(&self) -> bool {
        self.max.x < self.min.x || self.max.y < self.min.y || self.max.z < self.min.z
    }

    /// Returns the center point of the bounding box.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the size (dimensions) of the bounding box.
    pub fn size(&self) -> Vec3 {
        if self.is_empty() {
            Vec3::ZERO
        } else {
            self.max - self.min
        }
    }

    /// Returns the volume of the bounding box.
    pub fn volume(&self) -> f32 {
        if self.is_empty() {
            0.0
        } else {
            let size = self.size();
            size.x * size.y * size.z
        }
    }

    //=========================================================================
    // Mutation
    //=========================================================================

    /// Sets the bounding box to be empty.
    pub fn make_empty(&mut self) {
        self.min = Vec3::splat(f32::INFINITY);
        self.max = Vec3::splat(f32::NEG_INFINITY);
    }

    /// Expands the bounding box to include the given point.
    pub fn expand_by_point(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Expands the bounding box to include all given points.
    pub fn expand_by_points(&mut self, points: &[Vec3]) {
        for &point in points {
            self.expand_by_point(point);
        }
    }

    /// Expands the bounding box to include another bounding box.
    pub fn expand_by_bbox(&mut self, other: &BBox) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    /// Expands the bounding box by a scalar value in all directions.
    pub fn expand_by_scalar(&mut self, scalar: f32) {
        self.min -= Vec3::splat(scalar);
        self.max += Vec3::splat(scalar);
    }

    /// Expands the bounding box by a vector value.
    pub fn expand_by_vector(&mut self, vector: Vec3) {
        self.min -= vector;
        self.max += vector;
    }

    /// Translates the bounding box by a given offset.
    pub fn translate(&mut self, offset: Vec3) {
        self.min += offset;
        self.max += offset;
    }

    //=========================================================================
    // Queries
    //=========================================================================

    /// Returns true if the bounding box contains the given point.
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Returns true if the bounding box contains another bounding box.
    pub fn contains_bbox(&self, other: &BBox) -> bool {
        self.min.x <= other.min.x
            && other.max.x <= self.max.x
            && self.min.y <= other.min.y
            && other.max.y <= self.max.y
            && self.min.z <= other.min.z
            && other.max.z <= self.max.z
    }

    /// Returns true if the bounding box intersects with another bounding box.
    pub fn intersects_bbox(&self, other: &BBox) -> bool {
        !(other.max.x < self.min.x
            || other.min.x > self.max.x
            || other.max.y < self.min.y
            || other.min.y > self.max.y
            || other.max.z < self.min.z
            || other.min.z > self.max.z)
    }

    /// Returns the closest point within the bounding box to the given point.
    pub fn clamp_point(&self, point: Vec3) -> Vec3 {
        point.clamp(self.min, self.max)
    }

    /// Returns the distance from the bounding box to a point.
    /// If the point is inside the box, returns 0.
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        let clamped = self.clamp_point(point);
        clamped.distance(point)
    }

    //=========================================================================
    // Operations (returning new BBox)
    //=========================================================================

    /// Returns the intersection of this bounding box with another.
    pub fn intersect(&self, other: &BBox) -> BBox {
        BBox {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    /// Returns the union of this bounding box with another.
    pub fn union(&self, other: &BBox) -> BBox {
        BBox {
            min: self.min.min(other.min),
            max: self.max.min(other.max),
        }
    }

    /// Returns a new bounding box translated by the given offset.
    pub fn translated(&self, offset: Vec3) -> BBox {
        BBox {
            min: self.min + offset,
            max: self.max + offset,
        }
    }
}

impl Default for BBox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bbox_is_empty() {
        let bbox = BBox::new();
        assert!(bbox.is_empty());
    }

    #[test]
    fn test_expand_by_point() {
        let mut bbox = BBox::new();
        bbox.expand_by_point(Vec3::new(1.0, 2.0, 3.0));
        bbox.expand_by_point(Vec3::new(-1.0, -2.0, -3.0));

        assert_eq!(bbox.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(bbox.max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_center_and_size() {
        let bbox = BBox::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));

        assert_eq!(bbox.center(), Vec3::ZERO);
        assert_eq!(bbox.size(), Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_contains_point() {
        let bbox = BBox::from_min_max(Vec3::ZERO, Vec3::ONE);

        assert!(bbox.contains_point(Vec3::new(0.5, 0.5, 0.5)));
        assert!(!bbox.contains_point(Vec3::new(2.0, 2.0, 2.0)));
    }

    #[test]
    fn test_intersects_bbox() {
        let bbox1 = BBox::from_min_max(Vec3::ZERO, Vec3::ONE);
        let bbox2 = BBox::from_min_max(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
        let bbox3 = BBox::from_min_max(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));

        assert!(bbox1.intersects_bbox(&bbox2));
        assert!(!bbox1.intersects_bbox(&bbox3));
    }

    #[test]
    fn test_volume() {
        let bbox = BBox::from_min_max(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0));
        assert_eq!(bbox.volume(), 24.0);

        let empty_bbox = BBox::new();
        assert_eq!(empty_bbox.volume(), 0.0);
    }
}
