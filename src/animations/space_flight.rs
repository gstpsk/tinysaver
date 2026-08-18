use wgpu;

use crate::{animation::Animation, drawable::{Drawable, Material, Shape}, renderer::RenderContext, utils};
use crate::renderer::{Renderer2D, InstanceBatch};

const NUMBER_OF_STARS: u32 = 5000;
const STAR_SPEED: f32 = 0.25;

struct Star {
    shape: Drawable,
    z: f32, // 0.0 close, 1.0 is far
}

pub struct SpaceFlightAnimation {
    renderer: Renderer2D,
    drawables: Vec<Star>,
}

impl SpaceFlightAnimation {
    pub fn new(ctx: &RenderContext) -> Self {
        let renderer = Renderer2D::new(ctx);

        let mut drawables: Vec<Star> = Vec::new();

        let rect = Shape::Rectangle {
            width: 1.0,
            height: 1.0,
        };

        for _ in 0..NUMBER_OF_STARS {
            let (x, y) = utils::get_random_position(0, ctx.config.width - rect.width() as u32, 0, ctx.config.height - rect.height() as u32);
            
            let z = 1.0 - (rand::random::<f32>() % 0.95);

            let alpha = 1.0 - z;
            let alpha_u8 = (255.0 * alpha) as u8;
            let mut color: (u8, u8, u8) = (255, 255, 255);

            // get a 5% chance of a random blue/red shift
            let should_shift = rand::random_bool(0.05);

            if should_shift {
                let shift = rand::random::<u8>() % 150;
                // 50/50 chance
                if rand::random_bool(0.5) {
                    color.0 = 255-shift;
                    color.1 = 255-shift;
                    color.2 = 255;
                } else {
                    color.0 = 255;
                    color.1 = 255-shift;
                    color.2 = 255-shift;
                }

            }

            //let shape = ShapeDrawable::new(device, &renderer, rect, x as f32, y as f32, color);
            let shape = Drawable::new(rect, x as f32, y as f32, color, alpha_u8, Material::Solid);
            
            let star = Star {
                shape,
                z,
            };         
            
            drawables.push(star);
        }


        Self {
            renderer,
            drawables,        
        }
    }

    pub fn update(&mut self, ctx: &RenderContext) {
        self.update_position(ctx);
        self.update_appearance();
    }

    fn update_position(&mut self, ctx: &RenderContext) {
        // compute center
        let cx = ctx.config.width as f32 / 2.0;
        let cy = ctx.config.height as f32 / 2.0;

        for star in &mut self.drawables {
            // vector pointing to drawable
            let dx = star.shape.x - cx;
            let dy = star.shape.y - cy;

            let len = (dx*dx + dy*dy).sqrt();
            let dir_x = dx / len;
            let dir_y = dy / len;

            // move outward
            let speed = STAR_SPEED / star.z;
            star.shape.x += dir_x * speed;
            star.shape.y += dir_y * speed;

            // respawn if off-screen
            if star.shape.x < 0.0 || star.shape.x > ctx.config.width as f32 ||
            star.shape.y < 0.0 || star.shape.y > ctx.config.height as f32 {

                let (rx, ry) = utils::get_random_position(
                    0,
                    ctx.config.width,
                    0,
                    ctx.config.height
                );

                star.shape.x = rx as f32;
                star.shape.y = ry as f32;
            }

            //drawable.set_position(queue, drawable.x as u32, drawable.y as u32);
            star.shape.set_position(star.shape.x, star.shape.y);
        }
    }

    fn update_appearance(&mut self) {
        for star in &mut self.drawables {
            let alpha = 1.0 - star.z;
            let alpha_u8 = (255.0 * alpha) as u8;
            star.shape.set_alpha(alpha_u8);
            star.shape.set_scale(alpha*4.0, alpha*4.0);
        }
    }

    pub fn render(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.renderer.update_pixel_to_clip_buffer(ctx);

        let mut instance_batch = InstanceBatch {
            solid: Vec::with_capacity(self.drawables.len()),
            textured: Vec::new(),
            wireframe: Vec::new(),
        };

        for star in &self.drawables {
            let s = &star.shape;

            instance_batch.solid.push(s.to_instance_data());
        }

        self.renderer.upload_batches(&ctx.queue, &instance_batch);

        self.renderer.render(
            encoder,
            target,
            &instance_batch
        );
    }

}

impl Animation for SpaceFlightAnimation {
    fn update(&mut self, ctx: &RenderContext) {
        self.update(ctx);
    }

    fn render(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.render(ctx, encoder, target);
    }

    fn on_key(&mut self, key: winit::event::KeyEvent) {
        
    }
}