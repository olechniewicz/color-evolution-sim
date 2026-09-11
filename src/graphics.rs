use wgpu::*;
use std::mem;

pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: BindGroup,
    pub vertex_buffer: Buffer,
}

impl RenderPipeline {
    pub fn new(
        device: &Device,
        queue: &Queue,
        surface_format: TextureFormat,
        texture: &Texture,
    ) -> Self {
        // Load shader
        let shader_src = include_str!("shader.wgsl");
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("sim_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

        // Create sampler
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("sim_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // Create texture view
        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sim_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sim_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("sim_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sim_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        // Create full-screen quad vertex buffer (not used, but kept for structure)
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("quad_vertex_buffer"),
            size: 24,
            usage: BufferUsages::VERTEX,
            mapped_at_creation: true,
        });

        Self {
            pipeline,
            bind_group,
            vertex_buffer,
        }
    }
}
