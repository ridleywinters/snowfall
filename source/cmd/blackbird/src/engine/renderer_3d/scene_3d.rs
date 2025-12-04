use super::internal::*;
use super::triangle_buffer::TriangleBuffer;

pub struct Scene3D {
    pub camera: CameraPerspective,
    pub triangle_buffers: Vec<TriangleBuffer>,
}
