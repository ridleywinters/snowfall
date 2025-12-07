use super::internal::*;
use super::triangle_buffer::TriangleBuffer;
use crate::geometry::BBox;

pub struct Scene3D {
    pub camera: CameraPerspective,
    pub triangle_buffers: Vec<TriangleBuffer>,
}

impl Scene3D {
    pub fn bounding_box(&self) -> BBox {
        let mut bbox = BBox::new();
        for tb in &self.triangle_buffers {
            let b = tb.bounding_box();
            bbox.expand_by_bbox(&b);
        }
        bbox
    }
}
