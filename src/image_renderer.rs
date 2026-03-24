use pixels::wgpu::{self, ImageCopyTexture, util::DeviceExt};

use crate::drawable::Drawable;

// we use repr(C) to prevent Rust from messing with the memory layout
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // @location(0) position: vec2<f32>
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // @location(1) uv: vec2<f32>
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub offset: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Tint {
    pub color: [f32; 4],
}

pub struct Renderer2D {
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    pub projection_matrix_buffer: wgpu::Buffer,
    pub surface_width: u32,
    pub surface_height: u32
}

impl Renderer2D {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, surface_width: u32, surface_height: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("image renderer shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/image.wgsl").into()),
        });

        let sampler = Self::create_sampler(device);

        let projection_matrix_buffer = Self::create_projection_matrix_buffer(device, surface_width, surface_height);

        let bind_group_layout = Self::create_bind_group_layout(device);
                
        let render_pipeline_layout = Self::create_render_pipeline_layout(device, &bind_group_layout);
        
        let render_pipeline = Self::create_render_pipeline(device, &render_pipeline_layout, &shader, surface_format);

        Self {
            sampler,
            bind_group_layout,
            render_pipeline,
            projection_matrix_buffer,
            surface_width,
            surface_height
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, drawable: &impl Drawable) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("image renderer pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,       // keep what pixels already drew
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        drawable.bind(&mut render_pass);
        drawable.draw(&mut render_pass);
    }
    
    fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("yeah the sampler idk why label"),
            address_mode_u: wgpu::AddressMode::ClampToEdge, // if u bigger than 1 return right most pixel, and vice versa for if u smaller than 0
            address_mode_v: wgpu::AddressMode::ClampToEdge, // same thing but for v
            address_mode_w: wgpu::AddressMode::ClampToEdge, // needed but unused cause we have no w for 2d space
            mag_filter: wgpu::FilterMode::Nearest, // pixel perfect rendering, pick pixels nearest to (u, v) value
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest, // shouldnt matter as we dont use mipmaps
            lod_min_clamp: 0.0,                       // unused
            lod_max_clamp: 0.0,                       // unused
            compare: None,                            // unused
            anisotropy_clamp: 1,                      // unused
            border_color: None,                       // unused
        })
    }
    
    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let transform_binding_size = std::num::NonZeroU64::new(std::mem::size_of::<Transform>() as u64);
        let projection_matrix_binding_size = std::num::NonZeroU64::new(64); // 4x4 floats = 64 bytes
        let tint_color_binding_size = std::num::NonZeroU64::new(std::mem::size_of::<Tint>() as u64); // 4 floats = 16 bytes

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("amazing label for a bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,                                                             // bind it to 0
                    visibility: wgpu::ShaderStages::FRAGMENT,                               // visible to the fragment shader
                    ty: wgpu::BindingType::Texture {                                        // type of resource is texture
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },   // texture is floats because of texture format Rgba8UnormSrgb, and we allow filtering
                        view_dimension: wgpu::TextureViewDimension::D2,                     // 2D texture
                        multisampled: false,                                                // our image texture is not a msaa texture
                    },
                    count: None,                                                            // we are not using a texture array
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(                                         // this is the sampler so we use a different type
                        wgpu::SamplerBindingType::Filtering                                 // filtering sampler
                    ),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,                                                             // transformations
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: transform_binding_size
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,                                                             // projection matrix
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: projection_matrix_binding_size
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,                                                             // tint color
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: tint_color_binding_size
                    },
                    count: None,
                }
                ],
            })
    }

    fn create_render_pipeline_layout(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("epic render pipeline layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        })
    }

    fn create_render_pipeline(device: &wgpu::Device, render_pipeline_layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule, surface_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("epic render pipeline"),
            layout: Some(render_pipeline_layout),
            vertex: wgpu::VertexState { module: shader, entry_point: "vs_main", buffers: &[Vertex::desc()] },
            fragment: Some(wgpu::FragmentState { 
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // every three vertices correspond to a triangle
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,                                    // used to skip drawing triangles that face away, not really logical for 2d images, None draws everything
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,    // not used
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }

    pub fn create_transform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let initial_transform = Transform {
            offset: [0.0, 0.0], // initalise at the top left corner
        };

        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("initial transform buffer"),
                contents: bytemuck::bytes_of(&initial_transform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        )
    }

    pub fn create_tint_color_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let neutral_tint = Tint { color: [1.0, 1.0, 1.0, 1.0] }; // multiply by 1 so no effect on RGBA values

        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("neutral tint color buffer"),
                contents: bytemuck::bytes_of(&neutral_tint),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        )
    }

    fn create_projection_matrix_buffer(device: &wgpu::Device, surface_width: u32, surface_height: u32) -> wgpu::Buffer {
        let w = surface_width as f32;
        let h = surface_height as f32;
        let projection_matrix: [[f32; 4]; 4] = [
            // column 0
            [ 2.0 / w, 0.0,      0.0, 0.0 ],
            // column 1
            [ 0.0,    -2.0 / h,  0.0, 0.0 ],
            // column 2
            [ 0.0,     0.0,      1.0, 0.0 ],
            // column 3
            [ -1.0,    1.0,      0.0, 1.0 ],
        ];


        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("initial transform buffer"),
                contents: bytemuck::bytes_of(&projection_matrix),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        )
    }
}