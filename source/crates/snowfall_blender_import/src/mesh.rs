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

impl MScene {
    /// Compute the world-space bounding box of the entire scene
    pub fn scene_bounds(&self) -> BBox {
        let identity = MTransform {
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        };
        self.compute_bounds_recursive(&self.root.children, &identity)
    }

    fn compute_bounds_recursive(&self, nodes: &[MNode], parent_transform: &MTransform) -> BBox {
        let mut bounds = BBox::empty();

        for node in nodes {
            match node {
                MNode::MInstance(instance) => {
                    if let Some(mesh) = self.meshes.get(&instance.geometry_id) {
                        let transform = if let Some(t) = &instance.transform {
                            combine_transforms(parent_transform, t)
                        } else {
                            *parent_transform
                        };

                        let transformed_bbox = transform_bbox(&mesh.bbox, &transform);
                        bounds = bounds.merge(&transformed_bbox);
                    }
                }
                MNode::MGroup(group) => {
                    let transform = if let Some(t) = &group.transform {
                        combine_transforms(parent_transform, t)
                    } else {
                        *parent_transform
                    };

                    let child_bounds = self.compute_bounds_recursive(&group.children, &transform);
                    bounds = bounds.merge(&child_bounds);
                }
            }
        }

        bounds
    }
}

fn combine_transforms(parent: &MTransform, child: &MTransform) -> MTransform {
    MTransform {
        translation: parent.translation + child.translation * parent.scale,
        rotation: parent.rotation + child.rotation,
        scale: parent.scale * child.scale,
    }
}

fn transform_bbox(bbox: &BBox, transform: &MTransform) -> BBox {
    if bbox.is_empty() {
        return *bbox;
    }

    let corners = [
        Vec3::new(bbox.min.x, bbox.min.y, bbox.min.z),
        Vec3::new(bbox.max.x, bbox.min.y, bbox.min.z),
        Vec3::new(bbox.min.x, bbox.max.y, bbox.min.z),
        Vec3::new(bbox.max.x, bbox.max.y, bbox.min.z),
        Vec3::new(bbox.min.x, bbox.min.y, bbox.max.z),
        Vec3::new(bbox.max.x, bbox.min.y, bbox.max.z),
        Vec3::new(bbox.min.x, bbox.max.y, bbox.max.z),
        Vec3::new(bbox.max.x, bbox.max.y, bbox.max.z),
    ];

    let mut result = BBox::empty();
    for corner in &corners {
        let transformed = transform.translation + (*corner * transform.scale);
        if result.is_empty() {
            result = BBox::new(transformed, transformed);
        } else {
            result.min = result.min.min(transformed);
            result.max = result.max.max(transformed);
        }
    }

    result
}
