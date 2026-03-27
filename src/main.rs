//use std::{error::Error, process};

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey, SmolStr};
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

use font_kit::{family_name::FamilyName, font::Font, properties::Properties, source::SystemSource};
use pixels::{Pixels, SurfaceTexture, wgpu::Backend};

use crate::animation::Animation;
use crate::dvd_bounce::DvdBounceAnimation;
use crate::space_flight::SpaceFlightAnimation;
use crate::utils::load_image_rgba8;
//use crate::{dvd::DvdState, shader::SimpleShaderPass};

mod color;
mod draw;
//mod dvd;
mod shader;
mod test;
mod utils;
mod renderer;
mod image_drawable;
mod shape_drawable;
mod animation;
mod dvd_bounce;
mod space_flight;

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    animation: Option<Box<dyn Animation>>,
    frame_count: u32,
    fps: u32,
}

fn backend_to_str(backend: Backend) -> &'static str {
    // return static string slice
    match backend {
        Backend::Vulkan => "Vulkan",
        Backend::Metal => "Metal",
        Backend::Dx12 => "DirectX 12",
        Backend::Gl => "OpenGL",
        Backend::BrowserWebGpu => "WebGPU",
        _ => "Unknown",
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default().with_fullscreen(Some(Fullscreen::Borderless(None)));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        let monitors: Vec<_> = window.available_monitors().collect();
        let size = monitors.first().unwrap().size(); // this sucks but current/primary monitor returns None on wayland
        //let size = window.primary_monitor().unwrap().size();

        _ = window.request_inner_size(size);

        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        
        let mut pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();
        
        println!("GPU: {}", pixels.adapter().get_info().name);
        println!("Backend: {}", backend_to_str(pixels.adapter().get_info().backend));

        pixels.enable_vsync(false);

        //let shader = SimpleShaderPass::new(&pixels, size.width, size.height).unwrap();

        let (image_data, image_width, image_height) = load_image_rgba8("arch25percent.png");
        //let image_renderer = ImageRenderer::new(pixels.device(), pixels.queue(), image_width, image_height, &image_data, pixels.render_texture_format(), size.width, size.height);
        let dvd_bounce_animation = DvdBounceAnimation::new(pixels.device(), pixels.queue(), &image_data, image_width as i32, image_height as i32, pixels.render_texture_format(), size.width as i32, size.height as i32);
        let space_flight_animation = Box::new(SpaceFlightAnimation::new(pixels.device(), pixels.queue(), pixels.render_texture_format(), size.width as i32, size.height as i32));

        self.pixels = Some(pixels);
        self.animation = Some(space_flight_animation);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput { event, .. } => match event.logical_key {
                Key::Character(ref s) if s == "q" => event_loop.exit(),

                Key::Named(NamedKey::ArrowUp) => {
                    if let Some(animation) = &mut self.animation {
                        //animation.increase_speed_by(1);
                    }
                }

                Key::Named(NamedKey::ArrowDown) => {
                    if let Some(animation) = &mut self.animation {
                        //animation.decrease_speed_by(1);
                    }
                }

                _ => {}
            },

            WindowEvent::Resized(size) => {
                if let Some(px) = self.pixels.as_mut() {
                    if let Err(err) = px.resize_surface(size.width, size.height) {
                        eprintln!("resize_surface failed: {err}");
                        event_loop.exit();
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                let (Some(window), Some(pixels), Some(animation)) =
                    (&self.window, &mut self.pixels, &mut self.animation)
                else {
                    return;
                };

                let time_start = std::time::Instant::now();

                if self.frame_count % 200 == 0 {
                    println!("FPS: {}", self.fps);
                }

                animation.update(pixels.queue());
                
                if let Err(err) = pixels.render_with(|encoder, render_target, _context| {
                    //shader.render(encoder, target, pixels);
                    //image_renderer.render(encoder, render_target);
                    animation.render(encoder, render_target);
                    Ok(())
                }) {
                    eprintln!("pixels.render_with failed: {err}");
                    event_loop.exit();
                    return;
                }
                
                self.frame_count += 1;

                // Compute FPS
                self.fps = (std::time::Duration::from_secs(1).as_nanos() as f64
                    / time_start.elapsed().as_nanos() as f64) as u32;

                // Queue next frame
                window.request_redraw();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    
    let mut app = App {
        ..Default::default()
    };

    event_loop.run_app(&mut app)
}
