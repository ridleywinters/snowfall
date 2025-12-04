use super::internal::*;

pub struct CameraPerspective {
    // --- CPU-side data ---
    pub position: Vec3,
    pub look_at: Vec3,
    pub world_up: Vec3,

    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
    pub fovy_rad: f32,

    // --- WGPU bindings ---
    data: CameraData, // CPU-side data that needs to mirror the GPU layout
    buffer: Option<wgpu::Buffer>, // Handle to the GPU-side chunk of memory
}

impl CameraPerspective {
    //-----------------------------------------------------------------------//
    // Construction
    //-----------------------------------------------------------------------//

    pub fn new() -> Self {
        let data = CameraData::default();
        Self {
            position: Vec3::new(1.0, 2.0, 3.0),
            look_at: Vec3::ZERO,
            world_up: Vec3::Z,

            near: 0.1,
            far: 1000.0,
            aspect_ratio: 1.0,
            fovy_rad: 45.0_f32.to_radians(),

            data,
            buffer: None,
        }
    }

    //-----------------------------------------------------------------------//
    // Properties
    //-----------------------------------------------------------------------//

    pub fn view_proj(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.look_at, self.world_up);
        let proj = Mat4::perspective_rh(self.fovy_rad, self.aspect_ratio, self.near, self.far);
        proj * view
    }

    //-----------------------------------------------------------------------//
    // WGPU Bindings
    //-----------------------------------------------------------------------//

    pub fn wgsl_template(&self) -> &str {
        include_str!("camera_perspective.tmpl.wgsl")
    }

    pub fn update(&mut self) {
        let view = Mat4::look_at_rh(self.position, self.look_at, self.world_up);
        let proj = Mat4::perspective_rh(self.fovy_rad, self.aspect_ratio, self.near, self.far);

        self.data.position = Vec4::new(self.position.x, self.position.y, self.position.z, 1.0);
        self.data.view_proj = proj * view;
        self.data.inv_proj = proj.inverse();
        self.data.inv_view = view.inverse();

        let dir = Vec3::new(0.3, 0.7, -1.0).normalize();
        self.data.light_direction = Vec4::new(dir.x, dir.y, dir.z, 0.0);
    }

    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }]
    }

    pub fn bind_entries(&self) -> Vec<wgpu::BindingResource<'_>> {
        let buffer = self
            .buffer
            .as_ref()
            .expect("CameraPerspective buffer not created");
        vec![buffer.as_entire_binding()]
    }

    pub fn activate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let buffer = self.buffer.get_or_insert_with(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("CameraPerspective Uniform Buffer"),
                contents: bytemuck::cast_slice(&[self.data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        });
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[self.data]));
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraData {
    position: Vec4, // Using Vec4 for alignment
    view_proj: Mat4,
    inv_proj: Mat4,
    inv_view: Mat4,
    _padding: [f32; 4],
    light_direction: Vec4, // Using Vec4 for alignment
}
