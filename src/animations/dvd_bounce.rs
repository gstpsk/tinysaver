use crate::{animation::Animation, drawable::{Drawable, Material, Shape}, renderer::RenderContext, utils};
use crate::renderer::{Renderer2D, InstanceBatch};

use crate::color::Color;

pub struct DvdBounceAnimation {
    renderer: Renderer2D,
    drawable: Drawable,
    speed_x: f32,
    speed_y: f32,
    current_color: Color,
}

impl DvdBounceAnimation {
    pub fn new(
        ctx: &RenderContext,
        image_data: &[u8],
        image_width: u32,
        image_height: u32,
    ) -> Self {
        if image_width >= ctx.config.width || image_height >= ctx.config.height {
            panic!("Tried to create DvdBounceAnimation with too large image");
        }

        let mut renderer = Renderer2D::new(ctx);

        let texture = Renderer2D::create_texture_from_rgba8(
            &ctx.device,
            &ctx.queue,
            image_width as u32,
            image_height as u32,
            image_data,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shape = Shape::Rectangle { width: image_width as f32, height: image_height as f32 };
        let (x, y) = utils::get_random_position(ctx.config.width - image_width, ctx.config.height - image_height);

        let current_color = Color::random();
        
        let material = Material::Textured { 
            texture_index: renderer.add_texture_view(&ctx.device, texture_view)
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
        }
    }

    fn render(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.renderer.update_pixel_to_clip_buffer(ctx);

        let mut instance_batch = InstanceBatch {
            solid: Vec::new(),
            textured: Vec::with_capacity(1),
            wireframe: Vec::new()
        };

        instance_batch.textured.push(self.drawable.to_instance_data());

        self.renderer.upload_batches(&ctx.queue, &instance_batch);

        self.renderer.render(encoder, target, &instance_batch);
    }

    fn update_position(&mut self, ctx: &RenderContext) {
        // move
        self.drawable.x += self.speed_x as f32;
        self.drawable.y += self.speed_y as f32;

        // fix overshoot and bounce
        if self.handle_collision(ctx) {
            self.current_color = self.current_color.next();
            self.drawable.set_color(self.current_color.rgb());
        }
    }

    // invert speed if the image exceeds surface width after computation
    fn handle_collision(&mut self, ctx: &RenderContext) -> bool {
        let mut bounced = false;

        // right
        if self.drawable.x + self.drawable.shape.width() >= ctx.config.width as f32 {
            self.drawable.x = ctx.config.width as f32 - self.drawable.shape.width();
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
        if (self.drawable.y + self.drawable.shape.height() + self.speed_y) >= ctx.config.height as f32 {
            self.drawable.y = ctx.config.height as f32 - self.drawable.shape.height();
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

    fn increase_speed_by(&mut self, amount: f32) {
            if self.speed_x >= 0.0 { self.speed_x += amount; } else { self.speed_x -= amount; }
            if self.speed_y >= 0.0 { self.speed_y += amount; } else { self.speed_y -= amount; }
    }

    fn decrease_speed_by(&mut self, amount: f32) {
            if self.speed_x >= 0.0 { self.speed_x -= amount; } else { self.speed_x += amount; }
            if self.speed_y >= 0.0 { self.speed_y -= amount; } else { self.speed_y += amount; }
    }
}

impl Animation for DvdBounceAnimation {
    fn update(&mut self, ctx: &RenderContext) {
        self.update_position(ctx);
    }

    fn render(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.render(ctx, encoder, target);
    }

    fn on_key(&mut self, key: winit::event::KeyEvent) {

        
    }
}
