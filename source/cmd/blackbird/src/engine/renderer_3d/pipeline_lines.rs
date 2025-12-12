use crate::engine::prelude::CameraPerspective;

use super::internal::*;
use super::utils;

pub struct PipelineLines {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
}

impl PipelineLines {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        depth_format: wgpu::TextureFormat,
        camera: &mut CameraPerspective,
    ) -> Self {
        let mut shader_builder = ShaderSourceBuilder::new();
        shader_builder.source(include_str!("pipeline_lines.tmpl.wgsl"));
        shader_builder.mixin(camera.wgsl_template());
        let source = shader_builder.build("lines");
        shader_builder.log_to_file("pipeline_lines", &source);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Lines Shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        // Create the layout and the entries
        camera.prepare(&device);

        let (mut layouts, mut entries) = (vec![], vec![]);
        layouts.extend(camera.layout_entries());
        entries.extend(camera.bind_entries());

        let bind_group_layout = utils::create_bind_group_layout(device, layouts);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Lines Bind Group"),
            layout: &bind_group_layout,
            entries: &utils::create_bind_group_entries(entries),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Lines Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Lines Render Pipeline"),
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
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 1,
                    slope_scale: 1.0,
                    clamp: 0.0,
                },
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
