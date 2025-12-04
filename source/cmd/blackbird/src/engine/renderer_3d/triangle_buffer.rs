use super::internal::*;
use super::vertex::Vertex;

#[derive(Debug)]
pub struct TriangleBuffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl TriangleBuffer {
    //-----------------------------------------------------------------------//
    // Construction
    //-----------------------------------------------------------------------//

    pub fn new(
        device: &wgpu::Device,
        position_array: &Vec<Vec3>, //
        color_array: &Vec<Vec3>,    //
        index_array: &Vec<u32>,
    ) -> TriangleBuffer {
        let (vertices, indices) = {
            let mut v = Vec::new();
            for i in 0..position_array.len() {
                v.push(Vertex {
                    position: position_array[i].into(),
                    color: color_array[i].into(),
                });
            }
            let mut i = Vec::new();
            for j in 0..index_array.len() {
                i.push(index_array[j]);
            }
            (v, i)
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        TriangleBuffer {
            vertex_buffer,
            index_buffer,
        }
    }

    //-----------------------------------------------------------------------//
    // WGPU related
    //-----------------------------------------------------------------------//

    // Adds the commands to render the triangle buffer to the queue
    //
    pub fn activate(&self, render_pass: &mut wgpu::RenderPass) {
        if self.vertex_buffer.size() == 0 {
            panic!("TriangleBuffer has no vertices to render");
        }

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        let count = (self.index_buffer.size() / 4) as u32;
        if count > 0 {
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..count, 0, 0..1);
        } else {
            let vertex_count = self.vertex_buffer.size() / std::mem::size_of::<Vertex>() as u64;
            render_pass.draw(0..vertex_count as u32, 0..1);
        }
    }
}
