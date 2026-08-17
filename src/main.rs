use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::Key;
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

use crate::animation::Animation;
use crate::animations::{DvdBounceAnimation, RotatingCubeAnimation, SpaceFlightAnimation, WireframeAnimation};
use crate::renderer::RenderContext;
use crate::utils::load_image_rgba8;

mod animation;
mod animations;
mod drawable;
mod renderer;
mod utils;
mod color;

#[derive(Clone, Copy)]
enum AnimationType {
    Dvd,
    Space,
    Wireframe,
    RotatingCube,
}

impl AnimationType {
    fn from_string(string: &str) -> Option<Self> {
        match string {
            "dvd" => Some(Self::Dvd),
            "space" => Some(Self::Space),
            "wireframe" => Some(Self::Wireframe),
            "rotating-cube" => Some(Self::RotatingCube),
            _ => None,
        }
    }
}

const INVALID_ARGS: &str = "Please provide a valid animation selection.";
const USAGE: &str = "Usage: tinysaver [animation], choose from: dvd, space, wireframe.";

#[derive(Default)]
struct App {
    selected_animation: Option<AnimationType>,
    window: Option<Arc<Window>>,
    render_context: Option<RenderContext>,
    animation: Option<Box<dyn Animation>>,
    frame_count: u32,
    fps: u32,
}

fn backend_to_str(backend: wgpu::Backend) -> &'static str {
    // return static string slice
    match backend {
        wgpu::Backend::Vulkan => "Vulkan",
        wgpu::Backend::Metal => "Metal",
        wgpu::Backend::Dx12 => "DirectX 12",
        wgpu::Backend::Gl => "OpenGL",
        wgpu::Backend::BrowserWebGpu => "WebGPU",
        _ => "Unknown",
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes =
            WindowAttributes::default().with_fullscreen(Some(Fullscreen::Borderless(None)));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        //let monitors: Vec<_> = window.available_monitors().collect();
        //let size = monitors.first().unwrap().size(); // this sucks but current/primary monitor returns None on wayland
        //let size = window.primary_monitor().unwrap().size();

        //_ = window.request_inner_size(size);

        let ctx = RenderContext::new(window);

        println!("GPU: {}", ctx.adapter.get_info().name);
        println!(
            "Backend: {}",
            backend_to_str(ctx.adapter.get_info().backend)
        );

        println!(
            "window: {:?}, config: {}x{}",
            ctx.window.inner_size(),
            ctx.config.width,
            ctx.config.height,
        );

        //pixels.enable_vsync(false);

        //let shader = SimpleShaderPass::new(&pixels, size.width, size.height).unwrap();

        let (image_data, image_width, image_height) = load_image_rgba8("arch25percent.png");

        let animation: Box<dyn Animation> = match self.selected_animation.unwrap() {
            AnimationType::Dvd => Box::new(DvdBounceAnimation::new(
                &ctx,
                &image_data,
                image_width,
                image_height,
            )),
            AnimationType::Space => Box::new(SpaceFlightAnimation::new(
                &ctx
            )),
            AnimationType::Wireframe => Box::new(WireframeAnimation::new(
                &ctx
            )),
            AnimationType::RotatingCube => Box::new(RotatingCubeAnimation::new(
                &ctx
            ))
        };

        self.animation = Some(animation);
        self.render_context = Some(ctx);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                // On Q quit the application
                match event.logical_key {
                    Key::Character(ref s) if s == "q" => event_loop.exit(),
                    _ => ()
                }
                // Pass key events in their entirety
                if let Some(animation) = &mut self.animation {
                            animation.on_key(event);
                }
            }
            WindowEvent::Resized(size) => {
                println!("RESIZED: {}x{}", size.width, size.height);

                if let Some(render_context) = &mut self.render_context {
                    render_context.resize(size.width, size.height);
                } else {
                    return;
                };
            }
            WindowEvent::RedrawRequested => {
                let (Some(window), Some(render_context), Some(animation)) =
                    (&self.window, &mut self.render_context, &mut self.animation)
                else {
                    return;
                };

                let time_start = std::time::Instant::now();

                if self.frame_count % 200 == 0 {
                    println!("FPS: {}", self.fps);
                }

                animation.update(&render_context);

                render_context
                    .render(animation.as_mut())
                    .expect("Failed to render animation");

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

    let arg_string = std::env::args().nth(1);

    if arg_string.is_none() {
        println!("{}", INVALID_ARGS);
        println!("{}", USAGE);
        std::process::exit(1);
    }

    let selected_animation = AnimationType::from_string(&arg_string.unwrap());

    if selected_animation.is_none() {
        println!("{}", INVALID_ARGS);
        println!("{}", USAGE);
        std::process::exit(1);
    }

    let mut app = App {
        selected_animation,
        ..Default::default()
    };

    event_loop.run_app(&mut app)
}
