use pixels::wgpu::{self, ImageCopyTexture, util::DeviceExt};

// we use repr(C) to prevent Rust from messing with the memory layout
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
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
                    format: wgpu::VertexFormat::Float32x3,
                },
                // @location(1) uv: vec2<f32>
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
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

pub struct ImageRenderer {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    transform_buffer: wgpu::Buffer,
    projection_matrix_buffer: wgpu::Buffer,
    tint_color_buffer: wgpu::Buffer,
    surface_width: u32,
    surface_height: u32
}

impl ImageRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, rgba_data: &[u8], surface_format: wgpu::TextureFormat, surface_width: u32, surface_height: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("image renderer shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/image.wgsl").into()),
        });
        
        let texture = Self::create_texture_from_rgba8(device, queue, width, height, rgba_data);
        
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = Self::create_sampler(device);

        let transform_buffer = Self::create_transform_buffer(device);

        let projection_matrix_buffer = Self::create_projection_matrix_buffer(device, surface_width, surface_height);

        let tint_color_buffer = Self::create_tint_color_buffer(device);

        let bind_group_layout = Self::create_bind_group_layout(device);
        
        let bind_group = Self::create_bind_group(device, &bind_group_layout, &texture_view, &sampler, &transform_buffer, &projection_matrix_buffer, &tint_color_buffer);
        
        let render_pipeline_layout = Self::create_render_pipeline_layout(device, &bind_group_layout);
        
        let render_pipeline = Self::create_render_pipeline(device, &render_pipeline_layout, &shader, surface_format);

        let vertex_buffer = Self::create_vertex_buffer(device, width, height);

        let index_buffer = Self::create_index_buffer(device);

        Self {
            texture,
            texture_view,
            sampler,
            bind_group_layout,
            bind_group,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            transform_buffer,
            projection_matrix_buffer,
            tint_color_buffer,
            surface_width,
            surface_height
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
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
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // draw 6 indices
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    pub fn set_position(&self, queue: &wgpu::Queue, x: u32, y: u32) {
        let transform = Transform {
            offset: [x as f32, y as f32], // pixel coordinates directly
        };

        queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::bytes_of(&transform),
        );
    }

    pub fn set_tint_color(&self, queue: &wgpu::Queue, rgba: (u8, u8, u8, u8)) {
        // normalised and convert to float
        let tint_color_normalised = Tint {
            color: [rgba.0 as f32 / 255.0, rgba.1 as f32 / 255.0, rgba.2 as f32 / 255.0, rgba.3 as f32 / 255.0],
        };

        queue.write_buffer(&self.tint_color_buffer, 0, bytemuck::bytes_of(&tint_color_normalised));
    }

    fn create_texture_from_rgba8(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Image texture, why are there labels lol"),
            size: wgpu::Extent3d {
                // speaks for itself
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,                    // we only have 1 version
            sample_count: 1,                       // used for msaa apparently?
            dimension: wgpu::TextureDimension::D2, // 2 dimensions because images live in 2D
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, // texture binding means we can use it in the shader, copy_dst means we can copy something to it (write into it)
            view_formats: &[], // not needed...
        });

        queue.write_texture(
            ImageCopyTexture { 
                texture: &texture,                  // write to our newly created texture
                mip_level: 0,                       // write to the first mip level (we only have one)
                origin: wgpu::Origin3d::ZERO,       // begin writing at the start of texture
                aspect: wgpu::TextureAspect::All    // copy everything
            },
            data, 
            wgpu::ImageDataLayout {
                offset: 0,                          // start reading from the buffer at the begining
                bytes_per_row: Some(width * 4),     // each RGBA block is 4 bytes
                rows_per_image: Some(height),               
            }, 
            texture.size()        
        );

        texture
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
        
    fn create_bind_group(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, texture_view: &wgpu::TextureView, sampler: &wgpu::Sampler, transform_buffer: &wgpu::Buffer, projection_matrix_buffer: &wgpu::Buffer, tint_color_buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("epic bind group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler)
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(transform_buffer.as_entire_buffer_binding())
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(projection_matrix_buffer.as_entire_buffer_binding())
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(tint_color_buffer.as_entire_buffer_binding())
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

    fn create_vertex_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {        
        // create the quad to load the image texture into
        // and place the top left corner of the quad at the centre of the screen
        // use pixels first, then convert to clip space in the shader
        let vertices = [
            Vertex { position: [0.0, 0.0], uv: [0.0, 0.0] },                       // top left
            Vertex { position: [width as f32, 0.0], uv: [1.0, 0.0] },              // top right
            Vertex { position: [0.0, height as f32], uv: [0.0, 1.0] },             // bottom left
            Vertex { position: [width as f32, height as f32], uv: [1.0, 1.0] },    // bottom right
        ];

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("epic vertex buffer containg a quad"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn create_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        // two triangles make a quad
        let indices: [u16; 6] = [
            0, 1, 2,   // first triangle
            2, 1, 3,   // second triangle
        ];

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image renderer index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    fn create_transform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
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

    fn create_tint_color_buffer(device: &wgpu::Device) -> wgpu::Buffer {
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