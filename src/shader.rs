use pixels::{
    TextureError,
    wgpu::{self, util::DeviceExt},
};

pub struct SimpleShaderPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    vertex_buffer: wgpu::Buffer,
}

impl SimpleShaderPass {
    pub fn new(pixels: &pixels::Pixels, width: u32, height: u32) -> Result<Self, TextureError> {
        let device = pixels.device();
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/white.wgsl"));

        // Create a texture view that will be used as input
        // This will be used as the render target for the default scaling renderer
        let texture = create_texture(pixels, width, height)?;
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let vertices: [[f32; 2]; 3] = [
            // One full-screen triangle
            // See: https://github.com/parasyte/pixels/issues/180
            [-1.0, -1.0],
            [3.0, -1.0],
            [-1.0, 3.0],
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fullscreen triangle"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // bind group layout texture and sampler
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shader bind group layout"),
            entries: &[
                // texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("postprocess sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Use pixels own texture and sampler
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shader bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shader pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shader pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 8,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
                //compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: pixels.render_texture_format(),
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                //compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            //cache: None,
        });

        Ok(Self {
            pipeline,
            bind_group_layout,
            sampler,
            vertex_buffer,
        })
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        pixels: &pixels::Pixels,
    ) {
        let device = pixels.device();

        // Create a view of Pixels' internal framebuffer
        let src_view = pixels
            .texture()
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create bind group for this frame
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("postprocess bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("postprocess pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: render_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                //depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.draw(0..3, 0..1);
    }
}

// create a new GPU texture
fn create_texture(
    pixels: &pixels::Pixels,
    width: u32,
    height: u32,
) -> Result<wgpu::Texture, TextureError> {
    let device = pixels.device();
    pixels::check_texture_size(device, width, height)?;
    let texture_descriptor = wgpu::TextureDescriptor {
        label: None,
        size: pixels::wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: pixels.render_texture_format(),
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    Ok(device.create_texture(&texture_descriptor))
}
