use super::internal::*;
use super::triangle_buffer::TriangleBuffer;
use crate::geometry::BBox;

pub struct Scene3D {
    pub camera: CameraPerspective,
    pub triangle_buffers: Vec<TriangleBuffer>,
}

impl Scene3D {
    pub fn new() -> Scene3D {
        Scene3D {
            camera: CameraPerspective::new(),
            triangle_buffers: Vec::new(),
        }
    }

    pub fn bounding_box(&self) -> BBox {
        let mut bbox = BBox::new();
        for tb in &self.triangle_buffers {
            let b = tb.bounding_box();
            bbox.expand_by_bbox(&b);
        }
        bbox
    }

    //-------------------------------------------------------------------------
    // Mutation
    //-------------------------------------------------------------------------

    pub fn add(&mut self, triangle_buffer: TriangleBuffer) {
        self.triangle_buffers.push(triangle_buffer);
    }
}
