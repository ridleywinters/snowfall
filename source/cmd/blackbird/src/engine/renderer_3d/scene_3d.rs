use super::internal::*;
use super::line_buffer::LineBuffer;
use super::triangle_buffer::TriangleBuffer;
use crate::geometry::BBox;

pub struct Scene3D {
    pub camera: CameraPerspective,
    pub triangle_buffers: Vec<TriangleBuffer>,
    pub line_buffers: Vec<LineBuffer>,
}

impl Scene3D {
    pub fn new() -> Scene3D {
        Scene3D {
            camera: CameraPerspective::new(),
            triangle_buffers: Vec::new(),
            line_buffers: Vec::new(),
        }
    }

    pub fn bounding_box(&self) -> BBox {
        let mut bbox = BBox::new();
        for tb in &self.triangle_buffers {
            let b = tb.bounding_box();
            bbox.expand_by_bbox(&b);
        }
        for lb in &self.line_buffers {
            let b = lb.bounding_box();
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

    pub fn add_line_buffer(&mut self, line_buffer: LineBuffer) {
        self.line_buffers.push(line_buffer);
    }
}
