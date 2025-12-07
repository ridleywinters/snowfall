use crate::geometry::BBox;

use super::internal::*;
use super::vertex::Vertex;

#[derive(Debug)]
pub struct TriangleBuffer {
    position_array: Option<Vec<Vec3>>,
    color_array: Option<Vec<Vec3>>,
    index_array: Option<Vec<u32>>,

    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
}

impl TriangleBuffer {
    //-----------------------------------------------------------------------//
    // Construction
    //-----------------------------------------------------------------------//

    pub fn new(
        position_array: &Vec<Vec3>, //
        color_array: &Vec<Vec3>,    //
        index_array: &Vec<u32>,
    ) -> TriangleBuffer {
        TriangleBuffer {
            position_array: Some(position_array.clone()),
            color_array: Some(color_array.clone()),
            index_array: Some(index_array.clone()),

            vertex_buffer: None,
            index_buffer: None,
        }
    }

    //-----------------------------------------------------------------------//
    // Properties
    //-----------------------------------------------------------------------//

    pub fn bounding_box(&self) -> BBox {
        let position_array = self
            .position_array
            .as_ref()
            .expect("TriangleBuffer position array not set");

        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for pos in position_array.iter() {
            min = min.min(*pos);
            max = max.max(*pos);
        }

        BBox::from_min_max(min, max)
    }

    //-----------------------------------------------------------------------//
    // WGPU related
    //-----------------------------------------------------------------------//

    pub fn prepare(&mut self, device: &wgpu::Device) {
        if self.vertex_buffer.is_some() && self.index_buffer.is_some() {
            return;
        }

        let position_array = self.position_array.as_ref().unwrap();
        let color_array = self.color_array.as_ref().unwrap();
        let index_array = self.index_array.as_ref().unwrap();

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

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
    }

    // Adds the commands to render the triangle buffer to the queue
    //
    pub fn activate(&self, render_pass: &mut wgpu::RenderPass) {
        let vertex_buffer = self
            .vertex_buffer
            .as_ref()
            .expect("TriangleBuffer vertex buffer not created");
        let index_buffer = self
            .index_buffer
            .as_ref()
            .expect("TriangleBuffer index buffer not created");

        if vertex_buffer.size() == 0 {
            panic!("TriangleBuffer has no vertices to render");
        }

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

        let count = (index_buffer.size() / 4) as u32;
        if count > 0 {
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..count, 0, 0..1);
        } else {
            let vertex_count = vertex_buffer.size() / std::mem::size_of::<Vertex>() as u64;
            render_pass.draw(0..vertex_count as u32, 0..1);
        }
    }
}
