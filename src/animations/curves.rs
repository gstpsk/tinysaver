use crate::renderer::{InstanceBatch, Renderer2D};
use crate::{
    animation::Animation,
    color,
    drawable::{Drawable, Material, Shape},
    renderer::RenderContext,
    utils::get_random_position,
};

struct Bezier {
    p0: (i32, i32),
    p1: (i32, i32),
    p2: (i32, i32),
    p3: (i32, i32),
}

const TOTAL_STEPS: i32 = 1000;

impl Bezier {
    fn new(p0: (i32, i32), p1: (i32, i32), p2: (i32, i32), p3: (i32, i32)) -> Self {
        Self { p0, p1, p2, p3 }
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
    bezier: Bezier,
    current_step: i32,
}

impl CurvesAnimation {
    fn generate_control_points(&mut self, ctx: &RenderContext) {
        let mut points: Vec<Drawable> = Vec::new();

        let p0 = get_random_position(
            0,
            ctx.config.width / 2,
            ctx.config.height / 2,
            ctx.config.height,
        );
        let p1 = get_random_position(0, ctx.config.width / 2, 0, ctx.config.height / 2);
        let p2 = get_random_position(
            ctx.config.width / 2,
            ctx.config.width,
            ctx.config.height / 2,
            ctx.config.height,
        );
        let p3 = get_random_position(
            ctx.config.width / 2,
            ctx.config.width,
            0,
            ctx.config.height / 2,
        );

        let bezier = Bezier::new(
            (p0.0 as i32, p0.1 as i32),
            (p1.0 as i32, p1.1 as i32),
            (p2.0 as i32, p2.1 as i32),
            (p3.0 as i32, p3.1 as i32),
        );

        let control_point_rect = Shape::Rectangle {
                width: 12.0,
                height:12.0,
        };

        let control_point0_drawable = Drawable::new(
            control_point_rect,
            p0.0 as f32,
            p0.1 as f32,
            color::Color::Blue.to_rgb(),
            255,
            Material::Solid,
        );

        let control_point1_drawable = Drawable::new(
            control_point_rect,
            p1.0 as f32,
            p1.1 as f32,
            color::Color::Red.to_rgb(),
            255,
            Material::Solid,
        );

        let control_point2_drawable = Drawable::new(
            control_point_rect,
            p2.0 as f32,
            p2.1 as f32,
            color::Color::Red.to_rgb(),
            255,
            Material::Solid,
        );

        let control_point3_drawable = Drawable::new(
            control_point_rect,
            p3.0 as f32,
            p3.1 as f32,
            color::Color::Blue.to_rgb(),
            255,
            Material::Solid,
        );

        points.push(control_point0_drawable);
        points.push(control_point1_drawable);
        points.push(control_point2_drawable);
        points.push(control_point3_drawable);


        self.bezier = bezier;
        self.points = points;
    }

    pub fn new(ctx: &RenderContext) -> Self {
        let renderer = Renderer2D::new(ctx);

        let points: Vec<Drawable> = Vec::new();
        let bezier = Bezier {
            p0: (0, 0),
            p1: (0, 0),
            p2: (0, 0),
            p3: (0, 0),
        };

        let mut out = Self {
            renderer,
            points,
            bezier,
            current_step: 0,
        };

        out.generate_control_points(ctx);

        out
    }

    pub fn render(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
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

        self.renderer.render(encoder, target, &instance_batch);
    }
}

impl Animation for CurvesAnimation {
    fn update(&mut self, ctx: &RenderContext) {
        if self.current_step < TOTAL_STEPS {
            let t: f32 = self.current_step as f32 / TOTAL_STEPS as f32;

            let x = self.bezier.bx(t, ctx.config.width);
            let y = self.bezier.by(t, ctx.config.height);

            let point = Drawable::new(
                Shape::Rectangle {
                    width: 2.0,
                    height: 2.0,
                },
                x as f32,
                y as f32,
                color::Color::White.to_rgb(),
                255,
                Material::Solid,
            );

            self.points.push(point);

            self.current_step += 1;
        } else {
            self.generate_control_points(ctx);
            self.current_step = 0;
        }
    }

    fn render(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        self.render(ctx, encoder, target);
    }

    fn on_key(&mut self, key: winit::event::KeyEvent) {}
}
