use wgpu;

pub trait Animation {
    fn update(&mut self, queue: &wgpu::Queue);
    fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
}