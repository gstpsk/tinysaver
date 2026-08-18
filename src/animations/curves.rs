use crate::{animation::Animation, drawable::{Drawable, Material, Shape}, renderer::RenderContext, utils};
use crate::renderer::{Renderer2D, InstanceBatch};

struct Bezier {
    p0: (i32, i32),
    p1: (i32, i32),
    p2: (i32, i32), 
    p3: (i32, i32),
}

impl Bezier {
    fn new(p0: (i32, i32), p1: (i32, i32), p2: (i32, i32), p3: (i32, i32)) -> Self {
        Self {
            p0,
            p1,
            p2,
            p3,
        }            
    }

    fn bx(&self, t: f32, width: u32) -> i32 {
        let bernstein0 = (1.0 - t) * (1.0 - t) * (1.0 - t) * (self.p0.0 as f32 / width as f32);
        let bernstein1 = 3.0 * (1.0 - t) * (1.0 - t) * t * (self.p1.0 as f32 / width as f32);
        let bernstein2 = 3.0 * (1.0 - t) * t * t * (self.p2.0 as f32 / width as f32);
        let bernstein3 = t * t * t * (self.p3.0 as f32 / width as f32);
        let out = (bernstein0 + bernstein1 + bernstein2 + bernstein3) * width as f32;
        return out as i32;
    }

    fn by(&self, t: f32, height: u32) -> i32 {
        let bernstein0 = (1.0 - t) * (1.0 - t) * (1.0 - t) * (self.p0.1 as f32 / height as f32);
        let bernstein1 = 3.0 * (1.0 - t) * (1.0 - t) * t * (self.p1.1 as f32 / height as f32);
        let bernstein2 = 3.0 * (1.0 - t) * t * t * (self.p2.1 as f32 / height as f32);
        let bernstein3 = t * t * t * (self.p3.1 as f32 / height as f32);
        let out = (bernstein0 + bernstein1 + bernstein2 + bernstein3) * height as f32;
        return out as i32;
    }
}

pub struct CurvesAnimation {
    renderer: Renderer2D,
    points: Vec<Drawable>,
}

impl CurvesAnimation {
    pub fn new(ctx: &RenderContext) -> Self {
        let renderer = Renderer2D::new(ctx);

        let mut points: Vec<Drawable> = Vec::new();

        let rect = Shape::Rectangle {
            width: 1.0,
            height: 1.0,
        };

        let p0 = (200, 1000);
        let p1 = (300, 800);
        let p2 = (700, 1000);
        let p3 = (700, 800);

        let bezier = Bezier::new(p0, p1, p2, p3);

        let steps = 500;

        println!("Making {} points...", steps);

        for i in 0..steps {
            let t: f32 = i as f32 / steps as f32;

            let x = bezier.bx(t, ctx.config.width);
            let y = bezier.by(t, ctx.config.height);

            let alpha = 1.0;
            let alpha_u8 = (255.0 * alpha) as u8;
            let color: (u8, u8, u8) = (255, 255, 255);

            //let shape = ShapeDrawable::new(device, &renderer, rect, x as f32, y as f32, color);
            let point = Drawable::new(rect, x as f32, y as f32, color, alpha_u8, Material::Solid);       
            
            points.push(point);
        }


        Self {
            renderer,
            points,        
        }
    }

    pub fn render(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.renderer.update_pixel_to_clip_buffer(ctx);

        let mut instance_batch = InstanceBatch {
            solid: Vec::with_capacity(self.points.len()),
            textured: Vec::new(),
            wireframe: Vec::new(),
        };

        for point in &self.points {
            instance_batch.solid.push(point.to_instance_data());
            
        }

        self.renderer.upload_batches(&ctx.queue, &instance_batch);

        self.renderer.render(
            encoder,
            target,
            &instance_batch
        );
    }

}

impl Animation for CurvesAnimation {
    fn update(&mut self, ctx: &RenderContext) {
        
    }

    fn render(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.render(ctx, encoder, target);
    }

    fn on_key(&mut self, key: winit::event::KeyEvent) {
        
    }
}