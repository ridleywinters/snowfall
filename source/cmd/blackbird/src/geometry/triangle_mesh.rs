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
    // Mutation
    //-------------------------------------------------------------------------

    pub fn vertex_selection(&mut self) -> crate::geometry::VertexSelection<'_> {
        crate::geometry::VertexSelection::new(self)
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

pub fn make_debug_cube_mesh() -> TriangleMesh {
    let positions = [
        // Face (z = 1)
        ([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0]),
        ([1.0, -1.0, 1.0], [0.0, 0.0, 1.0]),
        ([1.0, 1.0, 1.0], [0.0, 0.0, 1.0]),
        ([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0]),
        ([1.0, 1.0, 1.0], [0.0, 0.0, 1.0]),
        ([-1.0, 1.0, 1.0], [0.0, 0.0, 1.0]),
        // Face (z = -1)
        ([1.0, -1.0, -1.0], [1.0, 1.0, 0.0]),
        ([-1.0, -1.0, -1.0], [1.0, 1.0, 0.0]),
        ([-1.0, 1.0, -1.0], [1.0, 1.0, 0.0]),
        ([1.0, -1.0, -1.0], [1.0, 1.0, 0.0]),
        ([-1.0, 1.0, -1.0], [1.0, 1.0, 0.0]),
        ([1.0, 1.0, -1.0], [1.0, 1.0, 0.0]),
        // Face (x = -1)
        ([-1.0, -1.0, -1.0], [0.0, 1.0, 1.0]),
        ([-1.0, -1.0, 1.0], [0.0, 1.0, 1.0]),
        ([-1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
        ([-1.0, -1.0, -1.0], [0.0, 1.0, 1.0]),
        ([-1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
        ([-1.0, 1.0, -1.0], [0.0, 1.0, 1.0]),
        // Face (x = 1)
        ([1.0, -1.0, 1.0], [1.0, 0.0, 0.0]),
        ([1.0, -1.0, -1.0], [1.0, 0.0, 0.0]),
        ([1.0, 1.0, -1.0], [1.0, 0.0, 0.0]),
        ([1.0, -1.0, 1.0], [1.0, 0.0, 0.0]),
        ([1.0, 1.0, -1.0], [1.0, 0.0, 0.0]),
        ([1.0, 1.0, 1.0], [1.0, 0.0, 0.0]),
        // Face (y = 1)
        ([-1.0, 1.0, 1.0], [0.0, 1.0, 0.0]),
        ([1.0, 1.0, 1.0], [0.0, 1.0, 0.0]),
        ([1.0, 1.0, -1.0], [0.0, 1.0, 0.0]),
        ([-1.0, 1.0, 1.0], [0.0, 1.0, 0.0]),
        ([1.0, 1.0, -1.0], [0.0, 1.0, 0.0]),
        ([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0]),
        // Face (y = -1)
        ([-1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
        ([1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
        ([1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
        ([-1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
        ([1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
        ([-1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
    ];

    let mut vertices = Vec::with_capacity(36);

    for (pos, col) in positions.iter() {
        vertices.push(MeshVertex {
            position: Vec3::new(pos[0], pos[1], pos[2]),
            color: Vec3::new(col[0], col[1], col[2]),
        });
    }

    TriangleMesh::new(vertices)
}
