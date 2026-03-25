use pixels::wgpu::{self};

use crate::{image_drawable::ImageDrawable, renderer::Renderer2D, utils};

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

    fn rgba(self) -> (u8, u8, u8, u8) {
        match self {
            Color::Red    => (255,   0,   0, 255),
            Color::Green  => (  0, 255,   0, 255),
            Color::Blue   => (  0,   0, 255, 255),
            Color::Yellow => (255, 255,   0, 255),
            Color::Cyan   => (  0, 255, 255, 255),
            Color::Purple => (255,   0, 255, 255),
            Color::White  => (255, 255, 255, 255)
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
    drawable: ImageDrawable,
    x: i32,
    y: i32,
    speed_x: i32,
    speed_y: i32,
    image_width: i32,
    image_height: i32,
    color: Color,
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

        let renderer = Renderer2D::new(
            device,
            surface_format,
            surface_width as u32,
            surface_height as u32,
        );
        let drawable = ImageDrawable::new(device, queue, &renderer, image_width as u32, image_height as u32, image_data);
        let (x, y) = utils::get_random_position(surface_width - image_width, surface_height - image_height);
        let speed_x = 1;
        let speed_y = 1;

        let color = Color::random();
        drawable.set_tint_color(queue, color.rgba());

        println!("Create DVD bounce animation at ({x}, {y})");

        Self {
            renderer,
            drawable,
            x,
            y,
            speed_x,
            speed_y,
            image_width,
            image_height,
            color,
            surface_width,
            surface_height,
        }
    }

    // feels a bit weird we have to update the position in two places
    // maybe this could be improved...
    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.update_position(queue);

        self.drawable.set_position(queue, self.x as u32, self.y as u32);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.renderer.render(encoder, target, &self.drawable);
    }

    // invert speed if the image exceeds surface width after computation
    fn handle_collision(&mut self) -> bool {
        let mut bounced = false;

        // right
        if self.x + self.image_width >= self.surface_width {
            self.x = self.surface_width - self.image_width;
            self.speed_x = -self.speed_x;
            bounced = true;
        }

        // left
        if self.x <= 0 {
            self.x = 0;
            self.speed_x = -self.speed_x;
            bounced = true;
        }

        // bottom wall
        if (self.y + self.image_height + self.speed_y) >= self.surface_height {
            self.y = self.surface_height - self.image_height;
            self.speed_y = -self.speed_y;
            bounced = true;
        }


        // bottom corners
        if (self.y + self.speed_y) <= 0 {
            self.y = 0;
            self.speed_y = -self.speed_y;
            bounced = true;
        }

        bounced
    }

    fn update_position(&mut self, queue: &wgpu::Queue) {
        // move
        self.x += self.speed_x;
        self.y += self.speed_y;

        // fix overshoot and bounce
        if self.handle_collision() {
            self.color = self.color.next();
            self.drawable.set_tint_color(queue, self.color.next().rgba());
        }
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
