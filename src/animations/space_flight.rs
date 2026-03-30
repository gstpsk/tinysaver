use wgpu;

use crate::{animation::Animation, drawable::{Drawable, Material, Shape}, utils};
use crate::renderer::{Renderer2D, InstanceBatch};

struct Star {
    shape: Drawable,
    z: f32, // 0.0 close, 1.0 is far
}

pub struct SpaceFlightAnimation {
    renderer: Renderer2D,
    drawables: Vec<Star>,
    surface_width: i32,
    surface_height: i32,
}

impl SpaceFlightAnimation {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        surface_width: i32,
        surface_height: i32,
    ) -> Self {
        let renderer = Renderer2D::new(
            device,
            queue,
            surface_format,
            surface_width as u32,
            surface_height as u32,
        );

        let mut drawables: Vec<Star> = Vec::new();

        let rect = Shape::Rectangle {
            width: 1.0,
            height: 1.0,
        };

        for _ in 0..10000 {
            let (x, y) = utils::get_random_position(surface_width - rect.width() as i32, surface_height - rect.height() as i32);
            
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
            surface_width,
            surface_height            
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.update_position();
        self.update_appearance();
    }

    fn update_position(&mut self) {
        // compute center
        let cx = self.surface_width as f32 / 2.0;
        let cy = self.surface_height as f32 / 2.0;

        for star in &mut self.drawables {
            // vector pointing to drawable
            let dx = star.shape.x - cx;
            let dy = star.shape.y - cy;

            let len = (dx*dx + dy*dy).sqrt();
            let dir_x = dx / len;
            let dir_y = dy / len;

            // move outward
            let speed = 1.0 / star.z;
            star.shape.x += dir_x * speed;
            star.shape.y += dir_y * speed;

            // respawn if off-screen
            if star.shape.x < 0.0 || star.shape.x > self.surface_width as f32 ||
            star.shape.y < 0.0 || star.shape.y > self.surface_height as f32 {

                let (rx, ry) = utils::get_random_position(
                    self.surface_width,
                    self.surface_height
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

    pub fn render(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let mut instance_batch = InstanceBatch {
            solid: Vec::with_capacity(self.drawables.len()),
            textured: Vec::new(),
        };

        for star in &self.drawables {
            let s = &star.shape;

            instance_batch.solid.push(s.to_instance_data());
        }

        self.renderer.upload_batches(queue, &instance_batch);

        self.renderer.render(
            encoder,
            target,
            &instance_batch
        );
    }

}

impl Animation for SpaceFlightAnimation {
    fn update(&mut self, queue: &wgpu::Queue) {
        self.update(queue);
    }

    fn render(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.render(queue, encoder, target);
    }
}