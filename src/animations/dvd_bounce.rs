use wgpu::{self, wgc::device::queue};

use crate::{animation::Animation, drawable::{Drawable, Material, Shape}, utils};
use crate::renderer::{Renderer2D, InstanceBatch};

#[derive(Copy, Clone)]
enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Purple,
    White
}

impl Color {
    fn random() -> Color {
        Color::Purple
    }

    fn rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Red    => (255,   0,   0),
            Color::Green  => (  0, 255,   0),
            Color::Blue   => (  0,   0, 255),
            Color::Yellow => (255, 255,   0),
            Color::Cyan   => (  0, 255, 255),
            Color::Purple => (255,   0, 255),
            Color::White  => (255, 255, 255)
        }
    }

    // gives us the ability to cycle through colors
    fn next(self) -> Color {
        match self {
            Color::Red    => Color::Green,
            Color::Green  => Color::Blue,
            Color::Blue   => Color::Yellow,
            Color::Yellow => Color::Cyan,
            Color::Cyan   => Color::Purple,
            Color::Purple => Color::White,
            Color::White  => Color::Red,
        }
    }
}

const COLORS: [Color; 7] = [
    Color::Red,
    Color::Green,
    Color::Blue,
    Color::Yellow,
    Color::Cyan,
    Color::Purple,
    Color::White,
];

pub struct DvdBounceAnimation {
    renderer: Renderer2D,
    drawable: Drawable,
    speed_x: f32,
    speed_y: f32,
    current_color: Color,
    surface_width: i32,
    surface_height: i32,
}

impl DvdBounceAnimation {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image_data: &[u8],
        image_width: i32,
        image_height: i32,
        surface_format: wgpu::TextureFormat,
        surface_width: i32,
        surface_height: i32,
    ) -> Self {
        if image_width >= surface_width || image_height >= surface_height {
            panic!("Tried to create DvdBounceAnimation with too large image");
        }

        if image_width < 0 || image_height < 0 || surface_width < 0 || surface_height < 0 {
            panic!("weird shit");
        }

        let mut renderer = Renderer2D::new(
            device,
            queue,
            surface_format,
            surface_width as u32,
            surface_height as u32,
        );

        let texture = Renderer2D::create_texture_from_rgba8(
            device,
            queue,
            image_width as u32,
            image_height as u32,
            image_data,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shape = Shape::Rectangle { width: image_width as f32, height: image_height as f32 };
        let (x, y) = utils::get_random_position(surface_width - image_width, surface_height - image_height);

        let current_color = Color::random();
        
        let material = Material::Textured { 
            texture_index: renderer.add_texture_view(device, texture_view)
        };
        
        let drawable = Drawable::new(shape, x as f32, y as f32, current_color.rgb(), 255, material);

        let speed_x = 1.0;
        let speed_y = 1.0;

        println!("Create DVD bounce animation at ({x}, {y})");

        Self {
            renderer,
            drawable,
            speed_x,
            speed_y,
            current_color,
            surface_width,
            surface_height,
        }
    }

    // feels a bit weird we have to update the position in two places
    // maybe this could be improved...
    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.update_position(queue);

        //self.drawable.set_position(queue, self.x as u32, self.y as u32);
    }

    pub fn render(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let mut instance_batch = InstanceBatch {
            solid: Vec::new(),
            textured: Vec::with_capacity(1),
        };

        let tex_index = match self.drawable.material {
            Material::Textured { texture_index } => texture_index,
            _ => 0,
        };

        instance_batch.textured.push(self.drawable.to_instance_data());

        self.renderer.upload_batches(queue, &instance_batch);

        self.renderer.render(encoder, target, &instance_batch);
    }

    // invert speed if the image exceeds surface width after computation
    fn handle_collision(&mut self) -> bool {
        let mut bounced = false;


        // right
        if self.drawable.x + self.drawable.shape.width() >= self.surface_width as f32 {
            self.drawable.x = self.surface_width as f32 - self.drawable.shape.width();
            self.speed_x = -self.speed_x;
            bounced = true;
        }

        // left
        if self.drawable.x <= 0.0 {
            self.drawable.x = 0.0;
            self.speed_x = -self.speed_x;
            bounced = true;
        }

        // bottom wall
        if (self.drawable.y + self.drawable.shape.height() + self.speed_y) >= self.surface_height as f32 {
            self.drawable.y = self.surface_height as f32 - self.drawable.shape.height();
            self.speed_y = -self.speed_y;
            bounced = true;
        }


        // bottom corners
        if (self.drawable.y + self.speed_y) <= 0.0 {
            self.drawable.y = 0.0;
            self.speed_y = -self.speed_y;
            bounced = true;
        }

        bounced
    }

    fn update_position(&mut self, queue: &wgpu::Queue) {
        // move
        self.drawable.x += self.speed_x as f32;
        self.drawable.y += self.speed_y as f32;

        // fix overshoot and bounce
        if self.handle_collision() {
            self.current_color = self.current_color.next();
            self.drawable.set_color(self.current_color.rgb());
        }
    }

    pub fn increase_speed_by(&mut self, amount: f32) {
            if self.speed_x >= 0.0 { self.speed_x += amount; } else { self.speed_x -= amount; }
            if self.speed_y >= 0.0 { self.speed_y += amount; } else { self.speed_y -= amount; }
    }

    pub fn decrease_speed_by(&mut self, amount: f32) {
            if self.speed_x >= 0.0 { self.speed_x -= amount; } else { self.speed_x += amount; }
            if self.speed_y >= 0.0 { self.speed_y -= amount; } else { self.speed_y += amount; }
    }
}

impl Animation for DvdBounceAnimation {
    fn update(&mut self, queue: &wgpu::Queue) {
        self.update(queue);
    }

    fn render(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.render(queue, encoder, target);
    }
}
