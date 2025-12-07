use super::internal::*;

#[derive(Debug, Clone)]
pub struct MeshVertex {
    pub position: Vec3,
    pub color: Vec3,
}

#[derive(Debug, Clone)]
pub struct TriangleMesh {
    pub vertices: Vec<MeshVertex>,
}

impl TriangleMesh {
    //-------------------------------------------------------------------------
    // Construction
    //-------------------------------------------------------------------------

    pub fn new(vertices: Vec<MeshVertex>) -> TriangleMesh {
        TriangleMesh { vertices }
    }
    //-------------------------------------------------------------------------
    // Selection
    //-------------------------------------------------------------------------

    pub fn vertex_selection(&mut self) -> crate::geometry::VertexSelection<'_> {
        crate::geometry::VertexSelection::new(self)
    }

    //-------------------------------------------------------------------------
    // Mutation
    //-------------------------------------------------------------------------

    pub fn translate(&mut self, tx: f32, ty: f32, tz: f32) {
        let t = Vec3::new(tx, ty, tz);
        for vertex in self.vertices.iter_mut() {
            vertex.position += t;
        }
    }

    //-------------------------------------------------------------------------
    // Conversion
    //-------------------------------------------------------------------------

    /// Converts the mesh to a TriangleBuffer for rendering.
    /// Since the mesh is un-indexed, this creates sequential indices.
    pub fn to_triangle_buffer(&self) -> crate::engine::renderer_3d::TriangleBuffer {
        let mut position_array = Vec::with_capacity(self.vertices.len());
        let mut color_array = Vec::with_capacity(self.vertices.len());
        let mut index_array = Vec::with_capacity(self.vertices.len());

        for (i, vertex) in self.vertices.iter().enumerate() {
            position_array.push(vertex.position);
            color_array.push(vertex.color);
            index_array.push(i as u32);
        }

        crate::engine::renderer_3d::TriangleBuffer::new(&position_array, &color_array, &index_array)
    }
}
