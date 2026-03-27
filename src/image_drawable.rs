use pixels::wgpu::{self, ImageCopyTexture};

use wgpu::util::DeviceExt;

use crate::renderer::{self, Renderer2D};

pub struct ImageDrawable {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub width: u32,
    pub height: u32,
    transform_buffer: wgpu::Buffer,
    tint_color_buffer: wgpu::Buffer,
}

impl ImageDrawable {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &Renderer2D,
        width: u32,
        height: u32,
        rgba_data: &[u8],
    ) -> Self {
        let texture = Self::create_texture_from_rgba8(device, queue, width, height, rgba_data);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let vertex_buffer = Self::create_vertex_buffer(device, width, height);
        let index_buffer = Self::create_index_buffer(device);

        let transform_buffer = Renderer2D::create_transform_buffer(device);
        let tint_color_buffer = Renderer2D::create_tint_color_buffer(device);

        let bind_group = renderer.create_bind_group(
            device,
            &texture_view,
            &transform_buffer,
            &tint_color_buffer,
        );

        Self {
            texture,
            texture_view,
            bind_group,
            vertex_buffer,
            index_buffer,
            width,
            height,
            transform_buffer,
            tint_color_buffer
        }
    }

    pub fn set_position(&self, queue: &wgpu::Queue, x: u32, y: u32) {
        let transform = renderer::Transform {
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
        let tint_color_normalised = renderer::Tint {
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

    fn create_vertex_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {        
        // create the quad to load the image texture into
        // and place the top left corner of the quad at the centre of the screen
        // use pixels first, then convert to clip space in the shader
        let vertices = [
            renderer::Vertex { position: [0.0, 0.0], uv: [0.0, 0.0] },                       // top left
            renderer::Vertex { position: [width as f32, 0.0], uv: [1.0, 0.0] },              // top right
            renderer::Vertex { position: [0.0, height as f32], uv: [0.0, 1.0] },             // bottom left
            renderer::Vertex { position: [width as f32, height as f32], uv: [1.0, 1.0] },    // bottom right
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
}

impl renderer::Drawable for ImageDrawable {
    fn pipeline_type(&self) -> renderer::PipelineType {
        renderer::PipelineType::Textured
    }

    fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
