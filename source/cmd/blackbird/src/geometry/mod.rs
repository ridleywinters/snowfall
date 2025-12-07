mod bbox;
mod triangle_mesh;
mod vertex_selection;

pub use bbox::BBox;
pub use triangle_mesh::{MeshVertex, TriangleMesh, make_debug_cube_mesh};
pub use vertex_selection::VertexSelection;

pub mod internal {
    pub use super::*;

    pub use glam::{Mat3, Mat4, Vec3, Vec4};
}
