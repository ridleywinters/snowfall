use super::internal::*;
use super::triangle_mesh::{MeshVertex, TriangleMesh};

#[derive(Debug)]
pub struct VertexSelection<'a> {
    mesh: &'a mut TriangleMesh,
    indices: Vec<usize>,
}

impl<'a> VertexSelection<'a> {
    //=========================================================================
    // Construction
    //=========================================================================

    pub fn new(mesh: &'a mut TriangleMesh) -> Self {
        Self {
            mesh,
            indices: Vec::new(),
        }
    }

    //=========================================================================
    // Selection Building
    //=========================================================================

    /// Adds vertices to the selection based on a predicate.
    pub fn add(mut self, predicate: impl Fn(&MeshVertex) -> bool) -> Self {
        for (i, vertex) in self.mesh.vertices.iter().enumerate() {
            if predicate(vertex) && !self.indices.contains(&i) {
                self.indices.push(i);
            }
        }
        self
    }

    pub fn all(mut self) -> Self {
        self.indices = (0..self.mesh.vertices.len()).collect();
        self
    }

    //=========================================================================
    // Queries
    //=========================================================================

    /// Returns the number of selected vertices.
    pub fn count(&self) -> usize {
        self.indices.len()
    }

    /// Returns true if the selection is empty.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Returns the bounding box of the selected vertices.
    pub fn bbox(&self) -> BBox {
        let mut bbox = BBox::new();
        for &idx in &self.indices {
            if let Some(vertex) = self.mesh.vertices.get(idx) {
                bbox.expand_by_point(vertex.position);
            }
        }
        bbox
    }

    /// Returns the center point of the selection.
    pub fn center(&self) -> Vec3 {
        self.bbox().center()
    }

    //=========================================================================
    // Transformations
    //=========================================================================

    /// Translates all selected vertices by the given offset.
    pub fn translate(self, tx: f32, ty: f32, tz: f32) -> Self {
        let t = Vec3::new(tx, ty, tz);
        for &idx in &self.indices {
            if let Some(vertex) = self.mesh.vertices.get_mut(idx) {
                vertex.position += t;
            }
        }
        self
    }

    /// Scales all selected vertices by the given factor relative to the selection center.
    pub fn scale(self, factor: Vec3) -> Self {
        let center = self.center();
        for &idx in &self.indices {
            if let Some(vertex) = self.mesh.vertices.get_mut(idx) {
                let offset = vertex.position - center;
                vertex.position = center + offset * factor;
            }
        }
        self
    }

    /// Scales all selected vertices uniformly by the given factor.
    pub fn scale_uniform(self, factor: f32) -> Self {
        self.scale(Vec3::splat(factor))
    }

    //=========================================================================
    // Attributes
    //=========================================================================

    /// Sets the color of all selected vertices.
    pub fn set_color(self, color: Vec3) -> Self {
        for &idx in &self.indices {
            if let Some(vertex) = self.mesh.vertices.get_mut(idx) {
                vertex.color = color;
            }
        }
        self
    }
}
