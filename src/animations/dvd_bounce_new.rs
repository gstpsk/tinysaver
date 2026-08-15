use crate::animation::Animation;
use crate::renderer::RenderContext;
use crate::renderer::Renderer3D;
use crate::renderer::Mesh;
use crate::renderer::GpuMesh;

pub struct DvdBounceAnimation2 {
    renderer: Renderer3D,
    rectangle: GpuMesh,
    model: glam::Mat4,
}

impl DvdBounceAnimation2 {
        pub fn new(ctx: &RenderContext) -> Self {
        let renderer = Renderer3D::new(ctx);

        let mesh = Mesh::cube();
        let rectangle = GpuMesh::from_mesh(&ctx.device, &mesh);
        let model = glam::Mat4::from_translation(glam::vec3(0.0, 0.0, 0.0));

        Self {
            renderer,
            rectangle,
            model,
        }
    }
}

impl Animation for DvdBounceAnimation2 {
    fn update(&mut self, ctx: &RenderContext) {
        self.renderer.update_transform(ctx, self.model);
    }

    fn render(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        self.renderer.render_mesh(encoder, target, &ctx.depth_texture.view, &self.rectangle, false, true);
    }

    fn on_key(&mut self, key: winit::event::KeyEvent) {
        
    }
}