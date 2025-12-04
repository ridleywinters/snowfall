mod camera_perspective;
mod create_device;
mod depth_texture;
mod pipeline_triangles;
mod renderer_3d;
mod scene_3d;
mod shader_source_builder;
mod triangle_buffer;
mod utils;
mod vertex;

pub use camera_perspective::CameraPerspective;
pub use renderer_3d::Renderer3D;
pub use scene_3d::Scene3D;

pub mod internal {
    pub use super::*;

    pub use glam::{Mat3, Mat4, Vec3, Vec4};
    pub use wgpu::util::DeviceExt;

    pub use super::pipeline_triangles::PipelineTriangles;
    pub use super::shader_source_builder::ShaderSourceBuilder;
    pub use super::triangle_buffer::TriangleBuffer;
    pub use super::vertex::Vertex;
}
