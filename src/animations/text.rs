use crate::{color, renderer::{InstanceData, RenderContext, diffuse_texture}};
use crate::renderer::{Renderer2D, InstanceBatch};

const GLYPH_WIDTH: u32 = 6;
const GLYPH_HEIGHT: u32 = 10;
const ATLAS_WIDTH: u32 = 78;
const ATLAS_HEIGHT: u32 = 70;
const CHARACTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-=()[]{}<>/*:#%!?.,'\"@&$ ";

struct Glyph {
    uv_min: [f32; 2],
    uv_max: [f32; 2],
}

impl Glyph {
    fn from_char(c: char) -> Self {
        // find index of the glyph
        let glyph_index = CHARACTERS.find(c).or_else(|| CHARACTERS.find('?')).unwrap() as u32;

        let columns = ATLAS_WIDTH / GLYPH_WIDTH;

        let column = glyph_index % columns;
        let row = glyph_index / columns;

        let x = column * GLYPH_WIDTH;
        let y = row * GLYPH_HEIGHT;

        Self {
            uv_min: [
                x as f32 / ATLAS_WIDTH as f32,
                y as f32 / ATLAS_HEIGHT as f32,
            ],
            uv_max: [
                (x + GLYPH_WIDTH) as f32 / ATLAS_WIDTH as f32,
                (y + GLYPH_HEIGHT) as f32 / ATLAS_HEIGHT as f32,
            ],
        }
    }
}

pub struct TextComponent {
    renderer: Renderer2D,
    texture_index: u32,
    text: String,
    scale: f32,
    color: color::Color,
}

impl TextComponent {
    pub fn new(ctx: &RenderContext, text: &str, scale: f32, color: color::Color) -> Self {

        let mut renderer = Renderer2D::new(ctx);

        let image = image::open("font.png").expect("Could not find fpf texture");
        let texture = diffuse_texture::create_diffuse_texture(ctx, Some("Text animation texture"), image);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_index = renderer.add_texture_view(&ctx.device, texture_view);

        Self {
            renderer,
            texture_index,
            text: text.to_string(),
            scale,
            color,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();        
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;        
    }

    pub fn set_color(&mut self, color: color::Color) {
        self.color = color;        
    }

    pub fn render(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.renderer.update_pixel_to_clip_buffer(ctx);

        let mut instance_batch = InstanceBatch {
            solid: Vec::new(),
            textured: Vec::with_capacity(self.text.len()),
            wireframe: Vec::new(),
        };

        let mut previous_line_length: f32 = 0.0;
        let mut current_row = 0.0;
        for (i, character) in self.text.chars().enumerate() {
            if character == '\n' {
                current_row += 1.0;
                previous_line_length = (i as f32) + 1.0;
                continue;
            }

            let glyph = Glyph::from_char(character);

            let index = i as f32;
            let x_offset = index - previous_line_length;
            let y_offset = current_row;

            let x = x_offset * self.scale as f32 * GLYPH_WIDTH as f32;
            let y = y_offset * self.scale as f32 * GLYPH_HEIGHT as f32;
            
            let instance = InstanceData {
                position: [x, y],
                size: [ // should maybe called "dimensions" instead of size
                    GLYPH_WIDTH as f32 * self.scale,
                    GLYPH_HEIGHT as f32 * self.scale as f32,
                ],
                rotation: 0.0,
                color: self.color.to_rgba_array(),
                texture_index: self.texture_index,
                instance_texture_uv_min: glyph.uv_min,
                instance_texture_uv_max: glyph.uv_max,
            };

            instance_batch.textured.push(instance);
        }

        self.renderer.upload_batches(&ctx.queue, &instance_batch);

        self.renderer.render(encoder, target, &instance_batch);
    }
}
