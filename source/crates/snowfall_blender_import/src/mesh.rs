use crate::BBox;
use glam::{Vec2, Vec3};

/// A mesh extracted from a Blender file.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub name: String,
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
    pub bbox: BBox,
}

impl Mesh {
    /// Create a new empty mesh with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            bbox: BBox::empty(),
        }
    }

    /// Get the number of vertices in this mesh
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Get the number of triangles in this mesh
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}
