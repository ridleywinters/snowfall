use std::collections::HashMap;

use super::internal::*;
use super::triangle_mesh::{MeshVertex, TriangleMesh};

pub struct MeshBuilder {
    library: HashMap<String, TriangleMesh>,
}

impl MeshBuilder {
    pub fn register(&mut self, name: &str, mesh: TriangleMesh) {
        self.library.insert(name.to_string(), mesh);
    }

    pub fn get(&self, name: &str) -> Option<TriangleMesh> {
        if let Some(mesh) = self.library.get(name) {
            Some(mesh.clone())
        } else {
            None
        }
    }

    pub fn make_unit_cube() -> TriangleMesh {
        let mut mesh = Self::make_debug_cube_mesh();
        mesh.vertex_selection()
            .all()
            .scale(Vec3::splat(0.5))
            .translate(0.5, 0.5, 0.5);
        mesh
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
}
