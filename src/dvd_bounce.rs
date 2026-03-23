use pixels::wgpu::{self, hal::empty::Encoder};

use crate::{image_renderer::ImageRenderer, utils};

pub struct DvdBounceAnimation {
    renderer: ImageRenderer,
    x: i32,
    y: i32,
    speed_x: i32,
    speed_y: i32,
    image_width: i32,
    image_height: i32,
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

        let renderer = ImageRenderer::new(
            device,
            queue,
            image_width as u32,
            image_height as u32,
            image_data,
            surface_format,
            surface_width as u32,
            surface_height as u32,
        );
        let (x, y) = utils::get_random_position(surface_width - image_width, surface_height - image_height);
        let speed_x = 1;
        let speed_y = 1;

        println!("Create DVD bounce animation at ({x}, {y})");

        Self {
            renderer,
            x,
            y,
            speed_x,
            speed_y,
            image_width,
            image_height,
            surface_width,
            surface_height,
        }
    }

    // feels a bit weird we have to update the position in two places
    // maybe this could be improved...
    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.update_position();

        self.renderer.set_position(queue, self.x as u32, self.y as u32);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.renderer.render(encoder, target);
    }

    // invert speed if the image exceeds surface width after computation
    fn handle_collision(&mut self) {
        // right
        if self.x + self.image_width >= self.surface_width {
            self.x = self.surface_width - self.image_width;
            self.speed_x = -self.speed_x;
        }

        // left
        if self.x <= 0 {
            self.x = 0;
            self.speed_x = -self.speed_x;
        }

        // bottom wall
        if (self.y + self.image_height + self.speed_y) >= self.surface_height {
            self.y = self.surface_height - self.image_height;
            self.speed_y = -self.speed_y;
        }


        // bottom corners
        if (self.y + self.speed_y) <= 0 {
            self.y = 0;
            self.speed_y = -self.speed_y;
        }
    }

    fn update_position(&mut self) {
        // move
        self.x += self.speed_x;
        self.y += self.speed_y;

        // fix overshoot and bounce
        self.handle_collision();
    }

    pub fn increase_speed_by(&mut self, amount: i32) {
            if self.speed_x >= 0 { self.speed_x += amount; } else { self.speed_x -= amount; }
            if self.speed_y >= 0 { self.speed_y += amount; } else { self.speed_y -= amount; }
    }

    pub fn decrease_speed_by(&mut self, amount: i32) {
            if self.speed_x >= 0 { self.speed_x -= amount; } else { self.speed_x += amount; }
            if self.speed_y >= 0 { self.speed_y -= amount; } else { self.speed_y += amount; }
    }
}
