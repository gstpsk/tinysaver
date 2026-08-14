use wgpu;

use crate::renderer::RenderContext;

pub trait Animation {
    fn update(&mut self, ctx: &RenderContext);
    fn render(&self, ctx: &RenderContext, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
    fn on_key(&mut self, key: winit::event::KeyEvent);
}