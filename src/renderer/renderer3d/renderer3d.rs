use wgpu::util::DeviceExt;

use crate::renderer::{RenderContext, diffuse_texture};
use crate::renderer::mesh::GpuMesh;
use crate::renderer::renderer3d::{TransformUniform, depth_texture};
use crate::renderer::vertex3d::Vertex3D;

pub struct Renderer3D {
    transform_bind_group: wgpu::BindGroup,
    transform_buffer: wgpu::Buffer,
    texture_bind_group: wgpu::BindGroup,
    solid_render_pipeline: wgpu::RenderPipeline,
    wireframe_render_pipeline: wgpu::RenderPipeline,
}

impl Renderer3D {
    fn create_transform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let transform_uniform = TransformUniform::new();
        //let mvp_matrix = glam::Mat4::IDENTITY.to_cols_array_2d();

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform uniform buffer"),
            contents: bytemuck::bytes_of(&transform_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_transform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let transform_uniform_binding_size = std::num::NonZeroU64::new(std::mem::size_of::<TransformUniform>() as u64);

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("MVP bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,                                                             // mvp matrix
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: transform_uniform_binding_size
                    },
                    count: None,
                },
                ],
            })
    }

    fn create_transform_bind_group(device: &wgpu::Device, bind_group_layout: &wgpu::BindGroupLayout, transform_buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("MVP bind group"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        transform_buffer.as_entire_buffer_binding(),
                    ),
                }]
        })
    }

    fn create_texture_bind_group_layout(ctx: &RenderContext) -> wgpu::BindGroupLayout {
        ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,                                                             // texture
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },   // texture is floats because of texture format Rgba8UnormSrgb, and we allow filtering
                        view_dimension: wgpu::TextureViewDimension::D2,                     // 2D texture
                        multisampled: false,                                                // our image texture is not a msaa texture
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,                                                             // sampler
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    fn create_texture_bind_group(ctx: &RenderContext, bind_group_layout: &wgpu::BindGroupLayout, texture_view: wgpu::TextureView, sampler: wgpu::Sampler) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler)
                },
            ]
        })
    }

    fn create_sampler(ctx: &RenderContext) -> wgpu::Sampler {
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        sampler
    }

    fn create_render_pipeline_layout(device: &wgpu::Device, mvp_bind_group_layout: &wgpu::BindGroupLayout, texture_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("epic render pipeline layout"),
            bind_group_layouts: &[Some(mvp_bind_group_layout), Some(texture_bind_group_layout)],
            immediate_size: 0,
        })
    }

    fn create_render_pipeline(device: &wgpu::Device, render_pipeline_layout: &wgpu::PipelineLayout, shader: &wgpu::ShaderModule, fragment_entry: &str, surface_format: wgpu::TextureFormat, is_solid: bool) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("epic 3D render pipeline"),
            layout: Some(render_pipeline_layout),
            vertex: wgpu::VertexState { module: shader, entry_point: Some("vs_main"), buffers: &[Vertex3D::desc()], compilation_options: Default::default() },
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
                cull_mode: None,
                polygon_mode: if is_solid { wgpu::PolygonMode::Fill } else { wgpu::PolygonMode::Line },
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState { 
                format: depth_texture::DepthTexture::DEPTH_FORMAT, 
                depth_write_enabled: Some(true), 
                depth_compare: Some(wgpu::CompareFunction::Less), 
                stencil: wgpu::StencilState::default(), 
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }


    pub fn new(ctx: &RenderContext) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("image renderer shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../shaders/3d.wgsl").into()),
        });

        let transform_buffer = Self::create_transform_buffer(&ctx.device);
        
        let transform_bind_group_layout = Self::create_transform_bind_group_layout(&ctx.device);
        let transform_bind_group = Self::create_transform_bind_group(&ctx.device, &transform_bind_group_layout, &transform_buffer);

        let texture_view = Self::load_textures(ctx);
        let sampler = Self::create_sampler(ctx);

        let texture_bind_group_layout = Self::create_texture_bind_group_layout(ctx);
        let texture_bind_group = Self::create_texture_bind_group(ctx, &texture_bind_group_layout, texture_view, sampler);
                
        let render_pipeline_layout = Self::create_render_pipeline_layout(&ctx.device, &transform_bind_group_layout, &texture_bind_group_layout);
        
        let solid_render_pipeline = Self::create_render_pipeline(&ctx.device, &render_pipeline_layout, &shader, "fs_main", ctx.config.format, true);

        let wireframe_render_pipeline = Self::create_render_pipeline(&ctx.device, &render_pipeline_layout, &shader, "fs_main", ctx.config.format, false);
        
        Self {
            transform_bind_group,
            transform_buffer,
            texture_bind_group,
            solid_render_pipeline,
            wireframe_render_pipeline
        }
    }

    pub fn load_textures(ctx: &RenderContext) -> wgpu::TextureView {
        let image = image::open("animal.jpg").expect("Failed to load image");
        let diffuse_texture = diffuse_texture::create_diffuse_texture(ctx, Some("epic texture"), image);
        //let diffuse_texture = diffuse_texture::create_dummy_texture(ctx);
        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        diffuse_texture_view
    }

    // expects a single type of instances
    pub fn render_mesh(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, depth_view: &wgpu::TextureView, mesh: &GpuMesh, wireframe_mode: bool, textured_mode: bool) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Renderer3D render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        
        if wireframe_mode {
            render_pass.set_pipeline(&self.wireframe_render_pipeline);
        } else {
            render_pass.set_pipeline(&self.solid_render_pipeline);
        }

        render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
    }

    pub fn update_transform(&self, ctx: &RenderContext, model: glam::Mat4) {
        //let test = glam::camera::rh::proj::
        let projection = glam::camera::rh::proj::vulkan::perspective(
            45.0_f32.to_radians(),
            ctx.config.width as f32 / ctx.config.height as f32,
            0.1,
            100.0,
        );


        let view = glam::Mat4::IDENTITY;
        let normal_matrix = model.inverse().transpose();

        let transform_uniform = TransformUniform {
            model: model.to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            normal_matrix: normal_matrix.to_cols_array_2d(),
        };

        ctx.queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::bytes_of(&transform_uniform),
        );
    }

}