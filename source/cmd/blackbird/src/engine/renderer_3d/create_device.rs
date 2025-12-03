use crate::engine::prelude::EngineWindow;

pub fn create_surface<'a>(window: EngineWindow) -> (wgpu::Instance, wgpu::Surface<'a>) {
    let backend_sets = vec![wgpu::Backends::PRIMARY, wgpu::Backends::SECONDARY];
    for backends in backend_sets {
        // The instance is the top-level wgpu connection to a specific backend: e.g.
        // Vulkan, Metal, DX12, WebGPU, etc.
        let desc = wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        };
        let instance = wgpu::Instance::new(&desc);

        // The surface where the rendering will go, in the case the window we've created.
        let surface = instance.create_surface(window.clone());
        if let Ok(surface) = surface {
            return (instance, surface);
        };
    }

    panic!("Could not create surface; no valid backends");
}

pub fn select_surface_format(surface_caps: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
    let present_modes: Vec<String> = surface_caps
        .present_modes
        .iter()
        .map(|f| format!("{:?}", f))
        .collect();
    println!("Available present modes: {}", present_modes.join(", "));

    let available_formats: Vec<String> = surface_caps
        .formats
        .iter()
        .map(|f| format!("{:?}", f))
        .collect();
    println!(
        "Available surface formats: {}",
        available_formats.join(", ")
    );

    let mut surface_format_candidate = surface_caps
        .formats
        .iter()
        .find(|f| *f == &wgpu::TextureFormat::Bgra8Unorm);
    if surface_format_candidate.is_none() {
        surface_format_candidate = surface_caps
            .formats
            .iter()
            .find(|f| *f == &wgpu::TextureFormat::Rgba8UnormSrgb);
    }
    if surface_format_candidate.is_none() {
        surface_format_candidate = surface_caps.formats.iter().find(|f| f.is_srgb());
    }

    surface_format_candidate
        .copied()
        .unwrap_or(surface_caps.formats[0])
}

pub async fn create_device(
    target_window: EngineWindow,
) -> (
    wgpu::Surface<'static>,
    wgpu::SurfaceConfiguration,
    wgpu::Device,
    wgpu::Queue,
) {
    println!("Initializing WGPU instance...");

    let size: winit::dpi::PhysicalSize<u32> = target_window.inner_size();

    let (instance, surface) = create_surface(target_window);

    //
    // The adapter and device then both act as implementation layers between the
    // backend and the renderable surface. The adapter can allow configuration between,
    // for example a high-end GPU or an integrated GPU on the host system.  The device
    // in term is implementation aligned to specific capability requests.  The queue
    // is a device specific command queue for sending commands.
    //
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let limits = adapter.limits();
    println!(
        "Max texture dimensions: 1D = {}, 2D = {}, 3D = {}",
        limits.max_texture_dimension_1d,
        limits.max_texture_dimension_2d,
        limits.max_texture_dimension_3d
    );
    println!(
        "Max texture array layers: {}",
        limits.max_texture_array_layers
    );
    println!(
        "Max sampled textures per shader stage: {}",
        limits.max_sampled_textures_per_shader_stage
    );
    println!(
        "Max samplers per shader stage: {}",
        limits.max_samplers_per_shader_stage
    );
    println!(
        "Max storage textures per shader stage: {}",
        limits.max_storage_textures_per_shader_stage
    );
    println!("Max bind groups: {}", limits.max_bind_groups);
    println!(
        "Max bindings per bind group: {}",
        limits.max_bindings_per_bind_group
    );
    println!(
        "Max dynamic uniform buffers per pipeline layout: {}",
        limits.max_dynamic_uniform_buffers_per_pipeline_layout
    );
    println!(
        "Max dynamic storage buffers per pipeline layout: {}",
        limits.max_dynamic_storage_buffers_per_pipeline_layout
    );
    println!(
        "Max uniform buffers per shader stage: {}",
        limits.max_uniform_buffers_per_shader_stage
    );
    println!(
        "Max storage buffers per shader stage: {}",
        limits.max_storage_buffers_per_shader_stage
    );

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            memory_hints: wgpu::MemoryHints::default(),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::default(),
        })
        .await
        .unwrap();

    // --- Surface & render pipeline configuration ---
    //
    // Now that the "hardware" is all set up, we need to do further configuration of
    // the surface we're rendering to and the render pipeline that's sending data
    // to that surface.
    //
    // Loosely, this configuration is more tied to how we want our rendering
    // engine to work whereas the prior configuration was more tied to what
    // we needed from the underlying hardware.
    //

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = select_surface_format(&surface_caps);
    println!("Using WGPU surface format {:?}", surface_format);

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    println!("Present mode: {:?}", surface_config.present_mode);

    surface.configure(&device, &surface_config);

    (surface, surface_config, device, queue)
}
