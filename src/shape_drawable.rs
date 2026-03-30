use core::panic;

use wgpu;

use wgpu::util::DeviceExt;

use crate::renderer::{self, Renderer2D};

#[derive(Copy, Clone)]
pub enum ShapeType {
    Rectangle { 
        width: f32, 
        height: f32 
    }
}

impl ShapeType {
    pub fn width(&self) -> f32 {
        match *self {
            ShapeType::Rectangle { width, .. } => width,
        }
    }

    pub fn height(&self) -> f32 {
        match *self {
            ShapeType::Rectangle { height, .. } => height,
        }
    }
}

pub struct ShapeDrawable {
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    transform_buffer: wgpu::Buffer,
    tint_color_buffer: wgpu::Buffer,
    pub x: f32,
    pub y: f32,
    color: (u8, u8, u8, u8),
    pub scale_x: f32,
    pub scale_y: f32
}

impl ShapeDrawable {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &Renderer2D,
        shape_type: ShapeType,
        x: f32,
        y: f32,
        color: (u8,u8,u8,u8),
    ) -> Self {
        //let texture = todo!();
        //let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let (vertex_buffer, index_buffer) = match shape_type {
            ShapeType::Rectangle { width, height } => {
                Self::create_rectangle_vertices(device, width, height)
            },
            _ => { panic!("Unknown shape type"); }
        };

        let transform_buffer = Renderer2D::create_transform_buffer(device);
        let tint_color_buffer = Renderer2D::create_tint_color_buffer(device);

        let bind_group = renderer.create_bind_group(
            device,
            &renderer.dummy_texture_view,
            &transform_buffer,
            &tint_color_buffer,
        );

        Self {
            bind_group,
            vertex_buffer,
            index_buffer,
            transform_buffer,
            tint_color_buffer,
            x,
            y,
            color,
            scale_x: 1.0,
            scale_y: 1.0
        }
    }

    pub fn pipeline_type(&self) -> renderer::PipelineType {
        renderer::PipelineType::Solid
    }

    pub fn set_position(&mut self, queue: &wgpu::Queue, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.update_transform_buffer(queue);
    }

    pub fn set_scale(&mut self, queue: &wgpu::Queue, scale_x: f32, scale_y: f32) {
        self.scale_x = scale_x;
        self.scale_y = scale_y;
        self.update_transform_buffer(queue);
    }

    pub fn set_color(&mut self, queue: &wgpu::Queue, rgb: (u8, u8, u8)) {
        self.color.0 = rgb.0;
        self.color.1 = rgb.1;
        self.color.2 = rgb.2;        
    }

    pub fn set_alpha(&mut self, queue: &wgpu::Queue, alpha: (u8)) {
        self.color.3 = alpha;
        self.update_color_buffer(queue);
    }

    fn update_transform_buffer(&self, queue: &wgpu::Queue) {
        let transform = renderer::Transform {
            offset: [self.x, self.y], // pixel coordinates directly
            scale: [self.scale_x, self.scale_y],
            rotation: 0.0,
            _padding: 0.0
        };

        queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::bytes_of(&transform),
        );
    }

    fn update_color_buffer(&self, queue: &wgpu::Queue) {
        // normalised and convert to float
        let color_normalised = renderer::Tint {
            color: [self.color.0 as f32 / 255.0, self.color.1 as f32 / 255.0, self.color.2 as f32 / 255.0, self.color.3 as f32 / 255.0],
        };

        queue.write_buffer(&self.tint_color_buffer, 0, bytemuck::bytes_of(&color_normalised));
    }

    fn create_rectangle_vertices(device: &wgpu::Device, width: f32, height: f32) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertices = [
            renderer::Vertex { position: [0.0, 0.0], uv: [0.0, 0.0] },                       // top left
            renderer::Vertex { position: [width as f32, 0.0], uv: [1.0, 0.0] },              // top right
            renderer::Vertex { position: [0.0, height as f32], uv: [0.0, 1.0] },             // bottom left
            renderer::Vertex { position: [width as f32, height as f32], uv: [1.0, 1.0] },    // bottom right
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
}

impl renderer::Drawable for ShapeDrawable {
    fn pipeline_type(&self) -> renderer::PipelineType {
        renderer::PipelineType::Solid
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