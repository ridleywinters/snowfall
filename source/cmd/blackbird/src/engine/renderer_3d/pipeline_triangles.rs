//! ✏️ NOTE: This was the "first" pipeline file written so it was written while conventions
//! were still being established.
//!
use crate::engine::prelude::CameraPerspective;

use super::internal::*;
use super::utils;

pub struct PipelineTriangles {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
}

impl PipelineTriangles {
    pub fn new(
        device: &wgpu::Device, //
        config: &wgpu::SurfaceConfiguration,
        depth_format: wgpu::TextureFormat,
        camera: &mut CameraPerspective,
    ) -> Self {
        let mut shader_builder = ShaderSourceBuilder::new();
        shader_builder.source(include_str!("pipeline_triangles.tmpl.wgsl"));
        let source = shader_builder.build("triangles");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        // Create the layout and the entries
        camera.prepare(&device);

        let (mut layouts, mut entries) = (vec![], vec![]);
        layouts.extend(camera.layout_entries());
        entries.extend(camera.bind_entries());

        let bind_group_layout = utils::create_bind_group_layout(device, layouts);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Triangles Bind Group"),
            layout: &bind_group_layout,
            entries: &utils::create_bind_group_entries(entries),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Triangles Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangles Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
        }
    }
}
