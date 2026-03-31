use crate::{animation::Animation, drawable::{self, Drawable}, renderer::{InstanceBatch, Renderer2D}};

pub struct WireframeAnimation {
    renderer: Renderer2D,
    drawables: Vec<Drawable>,
}

impl WireframeAnimation {
    pub fn new(
        device: &wgpu::Device, queue: &wgpu::Queue,
        wf_width: f32,
        wf_height: f32,
        surface_format: wgpu::TextureFormat,
        surface_width: u32,
        surface_height: u32,) -> Self {
            let renderer = Renderer2D::new(device, queue, surface_format, surface_width, surface_height);

            
            let center_x = (surface_width / 2) as f32;
            let center_y = (surface_height / 2) as f32;
            
            let drawable = Drawable::new(
                drawable::Shape::Rectangle { 
                    width:  wf_width, 
                    height: wf_height,
                }, 
                center_x - wf_width, 
                center_y - wf_height, 
                (255, 255, 255),
                255, 
                drawable::Material::Solid,
            );
            
            let drawable2 = Drawable::new(
                drawable::Shape::Rectangle { 
                    width:  wf_width, 
                    height: wf_height,
                }, 
                center_x - wf_width + wf_width * 2.0, 
                center_y - wf_height, 
                (255, 255, 255),
                255, 
                drawable::Material::Solid,
            );

            let drawables = vec![drawable, drawable2];

            Self {
                renderer,
                drawables
            }

    }

    pub fn render(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let mut instance_batch = InstanceBatch {
            solid: Vec::new(),
            textured: Vec::new(),
            wireframe: Vec::with_capacity(self.drawables.len())
        };

        for drawable in &self.drawables {
            instance_batch.wireframe.push(drawable.to_instance_data());
        }

        self.renderer.upload_batches(queue, &instance_batch);

        self.renderer.render(
            encoder,
            target,
            &instance_batch
        );
    }
}

impl Animation for WireframeAnimation {
    fn update(&mut self, queue: &wgpu::Queue) {
        //self.update(queue);
    }

    fn render(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        self.render(queue, encoder, target);
    }
}