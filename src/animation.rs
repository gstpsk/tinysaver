use wgpu;

pub trait Animation {
    fn update(&mut self, queue: &wgpu::Queue);
    fn render(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView);
}