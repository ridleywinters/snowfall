use crate::engine::prelude::EngineWindow;

pub struct Renderer3D {}

impl Renderer3D {
    pub fn new(window: EngineWindow) -> Self {
        // --- ⚠️ WARNING: Poll the future manually... ------------------------
        //
        // It feels a bit risky "hiding" a polling call in here, but it hides
        // the calling code from needing to worry about the async hand-off
        // between winit and wgpu. I'm new enough to Rust to not know how bad of
        // an idea this is!
        //
        let future = super::create_device::create_device(window);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (surface, surface_config, device, queue) = rt.block_on(future);

        let _ = super::depth_texture::DepthTexture::create_depth_texture(
            &device,
            surface_config.width,
            surface_config.height,
        );

        Self {}
    }
}
