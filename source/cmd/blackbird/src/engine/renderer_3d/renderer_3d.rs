use super::depth_texture::DepthTexture;
use super::internal::*;
use super::scene_3d::Scene3D;
use crate::engine::prelude::EngineWindow;

pub struct Renderer3D {
    // --- Device ---
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub depth_texture: DepthTexture,

    // --- Pipelines ---
    pub pipeline_triangles: Option<PipelineTriangles>,
    pub pipeline_lines: Option<PipelineLines>,
}

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

        let depth_texture = DepthTexture::create_depth_texture(
            &device,
            surface_config.width,
            surface_config.height,
        );

        Self {
            device,
            queue,
            surface,
            surface_config,
            depth_texture,

            pipeline_triangles: None,
            pipeline_lines: None,
        }
    }

    pub fn render_scene(&mut self, scene: &mut Scene3D) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let color_texture_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        scene.camera.update();

        for triangle_buffer in &mut scene.triangle_buffers {
            triangle_buffer.prepare(&self.device);
        }

        for line_buffer in &mut scene.line_buffers {
            line_buffer.prepare(&self.device);
        }

        run_render_pass(
            &mut encoder,
            &color_texture_view,
            &self.depth_texture,
            |pass| {
                if !scene.triangle_buffers.is_empty() {
                    let pipeline = self.pipeline_triangles.get_or_insert_with(|| {
                        let start_time = std::time::Instant::now();
                        let pipeline = PipelineTriangles::new(
                            &self.device,
                            &self.surface_config,
                            self.depth_texture.texture.format(),
                            &mut scene.camera,
                        );

                        println!(
                            "PipelineTriangles created in {} ms",
                            start_time.elapsed().as_millis()
                        );
                        pipeline
                    });
                    pass.set_pipeline(&pipeline.pipeline);
                    pass.set_bind_group(0, &pipeline.bind_group, &[]);

                    scene.camera.activate(&self.device, &self.queue);

                    for triangle_buffer in &scene.triangle_buffers {
                        triangle_buffer.activate(pass);
                    }
                }

                if !scene.line_buffers.is_empty() {
                    let pipeline = self.pipeline_lines.get_or_insert_with(|| {
                        let start_time = std::time::Instant::now();
                        let pipeline = PipelineLines::new(
                            &self.device,
                            &self.surface_config,
                            self.depth_texture.texture.format(),
                            &mut scene.camera,
                        );

                        println!(
                            "PipelineLines created in {} ms",
                            start_time.elapsed().as_millis()
                        );
                        pipeline
                    });
                    pass.set_pipeline(&pipeline.pipeline);
                    pass.set_bind_group(0, &pipeline.bind_group, &[]);

                    scene.camera.activate(&self.device, &self.queue);

                    for line_buffer in &scene.line_buffers {
                        line_buffer.activate(pass);
                    }
                }
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}

fn run_render_pass<'a, F>(
    encoder: &'a mut wgpu::CommandEncoder,
    color_texture_view: &'a wgpu::TextureView,
    depth_texture: &'a DepthTexture,
    f: F,
) where
    F: FnOnce(&mut wgpu::RenderPass<'a>),
{
    let desc = wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color_texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.02,
                    g: 0.03,
                    b: 0.08,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_texture.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    };

    let mut pass = encoder.begin_render_pass(&desc);
    f(&mut pass);
}
