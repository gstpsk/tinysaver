use winit::event::KeyEvent;
use winit::keyboard::Key;
use winit::keyboard::NamedKey;

use crate::animation::Animation;
use crate::renderer::RenderContext;
use crate::renderer::Renderer3D;
use crate::renderer::Mesh;
use crate::renderer::GpuMesh;

pub struct RotatingCubeAnimation {
    renderer: Renderer3D,
    cube: GpuMesh,
    model: glam::Mat4,
    x_rotation: bool,
    y_rotation: bool,
    z_rotation: bool,
    wireframe_mode: bool,
}

impl RotatingCubeAnimation {
        pub fn new(ctx: &RenderContext) -> Self {
        let renderer =
            Renderer3D::new(&ctx.device, ctx.config.format, ctx.config.width, ctx.config.height);

        // let color = (255, 255, 255);
        // let alpha = 255;

        let mesh = Mesh::cube();
        let cube = GpuMesh::from_mesh(&ctx.device, &mesh);
        let model = glam::Mat4::from_translation(glam::vec3(0.0, 0.0, -4.0));

        Self {
            renderer,
            cube,
            model,
            x_rotation: false,
            y_rotation: false,
            z_rotation: false,
            wireframe_mode: false,
        }
    }
}

impl Animation for RotatingCubeAnimation {
    fn update(&mut self, ctx: &RenderContext) {
        let rot_x = glam::Mat4::from_rotation_x(0.01);
        let rot_y = glam::Mat4::from_rotation_y(0.01);
        let rot_z = glam::Mat4::from_rotation_z(0.01);

        if self.x_rotation {
            self.model = self.model * rot_x;
        }
        
        if self.y_rotation {
            self.model = self.model * rot_y;
        }

        if self.z_rotation {
            self.model = self.model * rot_z;
        }

        self.renderer.update_transform(ctx, self.model);
    }

    fn render(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        self.renderer.render_mesh(encoder, target, &ctx.depth_texture.view, &self.cube, self.wireframe_mode);
    }

    fn on_key(&mut self, key: KeyEvent) {
        match key.logical_key {
            Key::Character(ref s) if s == "1" => {
                if key.state == winit::event::ElementState::Released {
                    self.x_rotation = !self.x_rotation;
                }
            },
            Key::Character(ref s) if s == "2" => {
                if key.state == winit::event::ElementState::Released {
                    self.y_rotation = !self.y_rotation;
                }
            },
            Key::Character(ref s) if s == "3" => {
                if key.state == winit::event::ElementState::Released {
                    self.z_rotation = !self.z_rotation;
                }
            },
            Key::Character(ref s) if s == "w" => { 
                let rot_x = glam::Mat4::from_rotation_x(-0.05);
                //let rot_y = glam::Mat4::from_rotation_y(0.01);

                self.model = self.model * rot_x //* rot_y;

                //self.renderer.update_mvp(queue, self.model);
            },
            Key::Character(ref s) if s == "a" => { 
                let rot_y = glam::Mat4::from_rotation_y(-0.05);
                //let rot_y = glam::Mat4::from_rotation_y(0.01);

                self.model = self.model * rot_y //* rot_y;

                //self.renderer.update_mvp(queue, self.model);
            },
            Key::Character(ref s) if s == "s" => { 
                let rot_x = glam::Mat4::from_rotation_x(0.05);
                //let rot_y = glam::Mat4::from_rotation_y(-0.01);

                self.model = self.model * rot_x //* rot_y;
            },
            Key::Character(ref s) if s == "d" => { 
                let rot_y = glam::Mat4::from_rotation_y(0.05);
                //let rot_y = glam::Mat4::from_rotation_y(0.01);

                self.model = self.model * rot_y //* rot_y;

                //self.renderer.update_mvp(queue, self.model);
            },
            Key::Named(NamedKey::Space) => {
                if key.state == winit::event::ElementState::Released {
                    self.wireframe_mode = !self.wireframe_mode;
                }
            },
            _ => {}
        }        
    }
}