use super::internal::*;
use super::triangle_buffer::TriangleBuffer;

pub fn create_bind_group_layout(
    device: &wgpu::Device,
    mut layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
) -> wgpu::BindGroupLayout {
    // Ensure the binding indices match what they will be in the layout
    //
    // ✏️ Note: I'm not sure if there's a "good reason" that WGPU has both
    // a binding index on the layout entry itself as well as an implied
    // index from the order of the entries.  It **seems** redundant which
    // is just an opportunity for things to get out of sync (bugs)...?
    //
    for (i, entry) in layout_entries.iter_mut().enumerate() {
        entry.binding = i as u32;
    }

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Render Bind Group Layout"),
        entries: &layout_entries,
    })
}

pub fn create_bind_group_entries(
    resources: Vec<wgpu::BindingResource>,
) -> Vec<wgpu::BindGroupEntry> {
    resources
        .iter()
        .enumerate()
        .map(|(i, resource)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: resource.clone(),
        })
        .collect::<Vec<_>>()
}

pub fn create_vertex_state<'a>(
    shader: &'a wgpu::ShaderModule,
    buffers: &'a [wgpu::VertexBufferLayout<'a>],
) -> Option<wgpu::VertexState<'a>> {
    Some(wgpu::VertexState {
        module: shader,
        entry_point: Some("vs_main"),
        buffers,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    })
}

pub fn make_full_screen_quad(device: &wgpu::Device) -> TriangleBuffer {
    let position_array = vec![
        Vec3::new(-1.0, -1.0, 0.0),
        Vec3::new(1.0, -1.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(-1.0, 1.0, 0.0),
    ];
    let color_array = vec![
        Vec3::new(0.0, 0.0, 0.5),
        Vec3::new(1.0, 0.0, 0.5),
        Vec3::new(1.0, 1.0, 0.5),
        Vec3::new(0.0, 1.0, 0.5),
    ];
    let index_array = vec![0, 1, 2, 2, 3, 0];

    TriangleBuffer::new(device, &position_array, &color_array, &index_array)
}

pub fn make_debug_cube(device: &wgpu::Device) -> TriangleBuffer {
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

    let mut position_array = Vec::with_capacity(36);
    let mut color_array = Vec::with_capacity(36);
    let mut index_array = Vec::with_capacity(36);

    for (i, (pos, normal)) in positions.iter().enumerate() {
        position_array.push(Vec3::new(pos[0], pos[1], pos[2]));
        color_array.push(Vec3::new(normal[0], normal[1], normal[2]));
        index_array.push(i as u32);
    }

    TriangleBuffer::new(device, &position_array, &color_array, &index_array)
}

#[derive(Debug, Clone)]
pub struct VertexAttrBuilder {
    attrs: Vec<wgpu::VertexAttribute>,
    offset: usize,
}

impl Default for VertexAttrBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl VertexAttrBuilder {
    pub fn new() -> Self {
        Self {
            attrs: Vec::new(),
            offset: 0,
        }
    }

    pub fn add_attr(mut self, format: wgpu::VertexFormat) -> Self {
        let shader_location = self.attrs.len() as u32;
        self.attrs.push(wgpu::VertexAttribute {
            offset: self.offset as wgpu::BufferAddress,
            shader_location,
            format,
        });
        self.offset += format.size() as usize;
        self
    }

    pub fn build(self) -> Vec<wgpu::VertexAttribute> {
        self.attrs
    }
}
