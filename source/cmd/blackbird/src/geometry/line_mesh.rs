use super::internal::*;

#[derive(Debug, Clone)]
pub struct LineMesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

impl LineMesh {
    //-------------------------------------------------------------------------
    // Construction
    //-------------------------------------------------------------------------

    /// Creates a WireframeMesh from a TriangleMesh by extracting edges.
    /// Each triangle edge becomes a line segment (with duplicates removed).
    pub fn from_triangle_mesh(mesh: &TriangleMesh) -> LineMesh {
        use std::collections::HashSet;

        let vertices = mesh.vertices.clone();
        let mut edges = HashSet::new();

        // Extract edges from triangles (every 3 vertices forms a triangle)
        for i in (0..mesh.vertices.len()).step_by(3) {
            if i + 2 >= mesh.vertices.len() {
                break;
            }

            let idx0 = i as u32;
            let idx1 = (i + 1) as u32;
            let idx2 = (i + 2) as u32;

            // Add three edges per triangle, normalized order to avoid duplicates
            edges.insert(if idx0 < idx1 {
                (idx0, idx1)
            } else {
                (idx1, idx0)
            });
            edges.insert(if idx1 < idx2 {
                (idx1, idx2)
            } else {
                (idx2, idx1)
            });
            edges.insert(if idx2 < idx0 {
                (idx2, idx0)
            } else {
                (idx0, idx2)
            });
        }

        // Flatten edges into line indices
        let mut indices = Vec::with_capacity(edges.len() * 2);
        for (a, b) in edges.iter() {
            indices.push(*a);
            indices.push(*b);
        }

        LineMesh { vertices, indices }
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

    pub fn remove_nonorthographic_lines(&mut self) {
        const EPS: f32 = 1e-6;
        let mut new_indices = Vec::with_capacity(self.indices.len());
        let verts = &self.vertices;

        for i in (0..self.indices.len()).step_by(2) {
            if i + 1 >= self.indices.len() {
                break;
            }
            let a = self.indices[i] as usize;
            let b = self.indices[i + 1] as usize;
            if a >= verts.len() || b >= verts.len() {
                continue;
            }

            let p0 = verts[a].position;
            let p1 = verts[b].position;
            let d = p1 - p0;

            let mut nonzero = 0;
            if d.x.abs() > EPS {
                nonzero += 1;
            }
            if d.y.abs() > EPS {
                nonzero += 1;
            }
            if d.z.abs() > EPS {
                nonzero += 1;
            }

            if nonzero == 1 {
                new_indices.push(self.indices[i]);
                new_indices.push(self.indices[i + 1]);
            }
        }

        self.indices = new_indices;
    }

    //-------------------------------------------------------------------------
    // Conversion
    //-------------------------------------------------------------------------

    /// Converts the mesh to a LineBuffer for rendering.
    pub fn to_line_buffer(&self) -> crate::engine::renderer_3d::LineBuffer {
        let mut position_array = Vec::with_capacity(self.vertices.len());
        let mut color_array = Vec::with_capacity(self.vertices.len());

        for vertex in self.vertices.iter() {
            position_array.push(vertex.position);
            color_array.push(vertex.color);
        }

        crate::engine::renderer_3d::LineBuffer::new(&position_array, &color_array, &self.indices)
    }
}
