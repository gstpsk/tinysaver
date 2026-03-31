use std::num::NonZeroU32;

use wgpu;
use wgpu::util::DeviceExt;

use crate::renderer::instance_data::{InstanceBatch, InstanceData};
use crate::renderer::vertex::Vertex;

pub const MAX_TEXTURES: u32 = 8;
pub const MAX_INSTANCES: u32 = 50000;

pub struct Renderer2D {
    texture_views: Vec<wgpu::TextureView>,
    pub next_texture_slot: u32,
    pub sampler: wgpu::Sampler,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group: wgpu::BindGroup,
    pub projection_bind_group_layout: wgpu::BindGroupLayout,
    pub projection_bind_group: wgpu::BindGroup,
    render_pipeline_solid: wgpu::RenderPipeline,
    render_pipeline_textured: wgpu::RenderPipeline,
    render_pipeline_wireframe: wgpu::RenderPipeline,
    pub quad_vertex_buffer: wgpu::Buffer,
    pub quad_index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub projection_matrix_buffer: wgpu::Buffer,
    pub surface_width: u32,
    pub surface_height: u32
}

impl Renderer2D {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_format: wgpu::TextureFormat, surface_width: u32, surface_height: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("image renderer shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/image.wgsl").into()),
        });

        let dummy_texture = Self::create_dummy_texture(device, queue);
        let dummy_texture_view = dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut texture_views = Vec::with_capacity(MAX_TEXTURES as usize);
        
        for _ in 0..MAX_TEXTURES {
            texture_views.push(dummy_texture_view.clone());
        }

        let next_texture_slot = 1; // 0 is dummy

        let texture_refs: Vec<&wgpu::TextureView> = texture_views.iter().collect();

        let sampler = Self::create_sampler(device);

        let projection_matrix_buffer = Self::create_projection_matrix_buffer(device, surface_width, surface_height);

        let texture_bind_group_layout = Self::create_texture_bind_group_layout(device);
        let texture_bind_group = Self::create_texture_bind_group(device, &texture_bind_group_layout, &texture_refs, &sampler);
        
        let projection_bind_group_layout = Self::create_projection_bind_group_layout(device);
        let projection_bind_group = Self::create_projection_bind_group(device, &projection_bind_group_layout, &projection_matrix_buffer);
                
        let render_pipeline_layout = Self::create_render_pipeline_layout(device, &texture_bind_group_layout, &projection_bind_group_layout);
        
        let render_pipeline_solid = Self::create_render_pipeline(device, &render_pipeline_layout, &shader, "fs_solid", surface_format);
        let render_pipeline_textured = Self::create_render_pipeline(device, &render_pipeline_layout, &shader, "fs_textured", surface_format);
        let render_pipeline_wireframe = Self::create_render_pipeline(device, &render_pipeline_layout, &shader, "fs_wireframe", surface_format);

        let (quad_vertex_buffer, quad_index_buffer) = Self::create_quad_vertices(device);

        let max_instances = MAX_INSTANCES as usize;
        
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buffer"),
            size: (max_instances * std::mem::size_of::<InstanceData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        Self {
            texture_views,
            next_texture_slot,
            sampler,
            texture_bind_group_layout,
            texture_bind_group,
            projection_bind_group_layout,
            projection_bind_group,
            render_pipeline_solid,
            render_pipeline_textured,
            render_pipeline_wireframe,
            quad_vertex_buffer,
            quad_index_buffer,
            instance_buffer,
            projection_matrix_buffer,
            surface_width,
            surface_height
        }
    }

    // expects a single type of instances
    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, instance_batch: &InstanceBatch) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("image renderer pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.projection_bind_group, &[]);
        // bind the quad vertex buffer
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        // bind the instance buffer
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // solid

        if !instance_batch.solid.is_empty() {
            render_pass.set_pipeline(&self.render_pipeline_solid);
            render_pass.draw_indexed(0..6, 0, 0..instance_batch.solid.len() as u32);

        }       

        if !instance_batch.textured.is_empty() {
            let textured_start = instance_batch.solid.len() as u32;
            let textured_end = textured_start + instance_batch.textured.len() as u32;
            render_pass.set_pipeline(&self.render_pipeline_textured);
            render_pass.draw_indexed(0..6, 0, textured_start..textured_end);
        }

        if !instance_batch.wireframe.is_empty() {
            let wireframe_start = (instance_batch.solid.len() + instance_batch.textured.len()) as u32;
            let wireframe_end = wireframe_start + instance_batch.wireframe.len() as u32;
            render_pass.set_pipeline(&self.render_pipeline_wireframe);
            render_pass.draw_indexed(0..6, 0, wireframe_start..wireframe_end);
        }

    }

    pub fn add_texture_view(&mut self, device: &wgpu::Device, texture_view: wgpu::TextureView) -> u32 {
        if self.next_texture_slot as usize >= self.texture_views.len() {
            panic!("Ran out of texture slots!");
        }
        
        self.texture_views[self.next_texture_slot as usize] = texture_view;
        self.next_texture_slot += 1;
        self.rebuild_texture_bind_group(device);
        return self.next_texture_slot-1 as u32;
    }


    fn rebuild_texture_bind_group(&mut self, device: &wgpu::Device) {
        let texture_refs: Vec<&wgpu::TextureView> =
            self.texture_views.iter().collect();

        self.texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rebuild texture bind group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&texture_refs),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
    }

    fn create_dummy_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let dummy_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dummy texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let white_pixel: [u8; 4] = [255, 255, 255, 255];

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &dummy_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &white_pixel,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        dummy_texture
    }
    
    fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("yeah the sampler idk why label"),
            address_mode_u: wgpu::AddressMode::ClampToEdge, // if u bigger than 1 return right most pixel, and vice versa for if u smaller than 0
            address_mode_v: wgpu::AddressMode::ClampToEdge, // same thing but for v
            address_mode_w: wgpu::AddressMode::ClampToEdge, // needed but unused cause we have no w for 2d space
            mag_filter: wgpu::FilterMode::Nearest, // pixel perfect rendering, pick pixels nearest to (u, v) value
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest, // shouldnt matter as we dont use mipmaps
            lod_min_clamp: 0.0,                       // unused
            lod_max_clamp: 0.0,                       // unused
            compare: None,                            // unused
            anisotropy_clamp: 1,                      // unused
            border_color: None,                       // unused
        })
    }
    
    fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
                    count: Some(NonZeroU32::new(MAX_TEXTURES).unwrap()),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(                                         // this is the sampler so we use a different type
                        wgpu::SamplerBindingType::Filtering                                 // filtering sampler
                    ),
                    count: None,
                },
                ],
            })
    }

    fn create_texture_bind_group(
        device: &wgpu::Device, 
        bind_group_layout: &wgpu::BindGroupLayout, 
        texture_views: &[&wgpu::TextureView], 
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(texture_views),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    fn create_projection_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let projection_matrix_binding_size = std::num::NonZeroU64::new(64); // 4x4 floats = 64 bytes

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("projection bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,                                                             // projection matrix
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: projection_matrix_binding_size
                    },
                    count: None,
                },
                ],
            })
    }

    fn create_projection_bind_group(
        device: &wgpu::Device, 
        bind_group_layout: &wgpu::BindGroupLayout,
        projection_matrix_buffer: &wgpu::Buffer
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("projection bind group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        projection_matrix_buffer.as_entire_buffer_binding(),
                    ),
                }]
            })
    }

    fn create_render_pipeline_layout(device: &wgpu::Device, texture_bind_group_layout: &wgpu::BindGroupLayout, projection_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("epic render pipeline layout"),
            bind_group_layouts: &[Some(texture_bind_group_layout), Some(projection_bind_group_layout)],
            immediate_size: 0,
        })
    }

    fn create_render_pipeline(device: &wgpu::Device, render_pipeline_layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule, fragment_entry: &str, surface_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("epic render pipeline"),
            layout: Some(render_pipeline_layout),
            vertex: wgpu::VertexState { module: shader, entry_point: Some("vs_main"), buffers: &[Vertex::desc(), InstanceData::desc()], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { 
                module: shader,
                entry_point: Some(fragment_entry),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            multiview_mask: None,
            cache: None,
        })
    }

    // pub fn create_transform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    //     let initial_transform = Transform {
    //         offset: [0.0, 0.0], // initalise at the top left corner
    //         scale: [1.0, 1.0],  // unscaled
    //         rotation: 0.0,      // no rotation
    //         _padding: 0.0,      // i know...
    //     };

    //     device.create_buffer_init(
    //         &wgpu::util::BufferInitDescriptor {
    //             label: Some("initial transform buffer"),
    //             contents: bytemuck::bytes_of(&initial_transform),
    //             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    //         }
    //     )
    // }

    // pub fn create_tint_color_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    //     let neutral_tint = Tint { color: [1.0, 1.0, 1.0, 1.0] }; // multiply by 1 so no effect on RGBA values

    //     device.create_buffer_init(
    //         &wgpu::util::BufferInitDescriptor {
    //             label: Some("neutral tint color buffer"),
    //             contents: bytemuck::bytes_of(&neutral_tint),
    //             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    //         }
    //     )
    // }

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

    fn create_quad_vertices(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertices = [
            Vertex { position: [0.0, 0.0], uv: [0.0, 0.0] },                       // top left
            Vertex { position: [1.0, 0.0], uv: [1.0, 0.0] },              // top right
            Vertex { position: [0.0, 1.0], uv: [0.0, 1.0] },             // bottom left
            Vertex { position: [1.0, 1.0], uv: [1.0, 1.0] },    // bottom right
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("epic vertex buffer containg a quad"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // two triangles make a quad
        let indices: [u16; 6] = [
            0, 1, 2,   // first triangle
            2, 1, 3,   // second triangle
        ];

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image renderer index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }

    pub fn upload_batches(&self, queue: &wgpu::Queue, instance_batch: &InstanceBatch) {
        // upload solid
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_batch.solid),
        );

        // upload textured
        let mut offset = (instance_batch.solid.len() * std::mem::size_of::<InstanceData>()) as u64;
        queue.write_buffer(
            &self.instance_buffer,
            offset,
            bytemuck::cast_slice(&instance_batch.textured),
        );

        // upload wireframe
        offset += (instance_batch.textured.len() * std::mem::size_of::<InstanceData>()) as u64;
        queue.write_buffer(
            &self.instance_buffer,
            offset,
            bytemuck::cast_slice(&instance_batch.wireframe),
        );
    }

    pub fn create_texture_from_rgba8(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Image texture from RGBA8"),
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
            wgpu::TexelCopyTextureInfo { 
                texture: &texture,                  // write to our newly created texture
                mip_level: 0,                       // write to the first mip level (we only have one)
                origin: wgpu::Origin3d::ZERO,       // begin writing at the start of texture
                aspect: wgpu::TextureAspect::All    // copy everything
            },
            data, 
            wgpu::TexelCopyBufferLayout {
                offset: 0,                          // start reading from the buffer at the begining
                bytes_per_row: Some(width * 4),     // each RGBA block is 4 bytes
                rows_per_image: Some(height),               
            }, 
            texture.size()        
        );

        texture
    }
}