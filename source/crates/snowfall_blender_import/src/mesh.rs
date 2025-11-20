use std::collections::HashMap;

use crate::BBox;
use glam::{Vec2, Vec3};

pub type MMeshID = String;
pub type MMaterialID = String;

#[derive(Debug, Clone)]
pub struct MMesh {
    pub id: MMeshID,
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
    pub bbox: BBox,
}

impl MMesh {
    pub fn new(name: String) -> Self {
        Self {
            id: name,
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

#[derive(Debug, Clone, Copy)]
pub struct MTransform {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

#[derive(Debug, Clone)]
pub enum MNode {
    MInstance(MInstance),
    MGroup(MGroup),
}

#[derive(Debug, Clone)]
pub struct MMaterial {}

#[derive(Debug, Clone)]
pub struct MInstance {
    pub geometry_id: MMeshID,
    pub material_id: Option<MMaterialID>,
    pub transform: Option<MTransform>,
}

#[derive(Debug, Clone)]
pub struct MGroup {
    pub children: Vec<MNode>,
    pub transform: Option<MTransform>,
}

#[derive(Debug, Clone)]
pub struct MScene {
    pub meshes: HashMap<MMeshID, MMesh>,
    pub materials: HashMap<MMaterialID, MMaterial>,
    pub root: MGroup,
}
