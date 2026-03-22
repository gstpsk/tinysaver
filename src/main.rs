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

use crate::{dvd::DvdState, shader::SimpleShaderPass};

mod color;
mod draw;
mod dvd;
mod shader;
mod test;
mod utils;

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    shader: Option<SimpleShaderPass>,
    dvd_state: Option<DvdState>,
    font: Option<Font>, // from font-kit
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

        let shader = SimpleShaderPass::new(&pixels, size.width, size.height).unwrap();

        self.pixels = Some(pixels);
        self.shader = Some(shader);
        self.dvd_state = Some(DvdState::with_random_position(
            1,
            size.width as i32,
            size.height as i32,
        ));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput { event, .. } => match event.logical_key {
                Key::Character(ref s) if s == "q" => event_loop.exit(),

                Key::Named(NamedKey::ArrowUp) => {
                    if let Some(state) = &mut self.dvd_state {
                        state.increase_speed_by(1);
                    }
                }

                Key::Named(NamedKey::ArrowDown) => {
                    if let Some(state) = &mut self.dvd_state {
                        state.decrease_speed_by(1);
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
                let (Some(window), Some(pixels), Some(dvd_state), Some(shader)) =
                    (&self.window, &mut self.pixels, &mut self.dvd_state, &mut self.shader)
                else {
                    return;
                };

                let time_start = std::time::Instant::now();

                let frame = pixels.frame_mut();

                // Draw FPS
                if let Some(font) = &self.font {
                    draw_fps(
                        frame,
                        self.frame_count,
                        self.fps,
                        font,
                        window.inner_size().width as i32,
                        window.inner_size().height as i32,
                    );
                }

                dvd::dvd_style(frame, self.frame_count, dvd_state);

                

                // Render
                pixels.render();
                /*
                if let Err(err) = pixels.render_with(|encoder, target, _context| {
                    //shader.render(encoder, target, pixels);
                    Ok(())
                }) {
                    eprintln!("pixels.render_with failed: {err}");
                    event_loop.exit();
                    return;
                }
                    */
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

fn draw_fps(
    frame: &mut [u8],
    frame_count: u32,
    fps: u32,
    font: &Font,
    surface_width: i32,
    surface_height: i32,
) {
    let black = (0, 0, 0, 0);
    let white = (255, 255, 255);
    if frame_count % 30 == 0 {
        draw::draw_rect(
            frame,
            100,
            100,
            500,
            200,
            0,
            None,
            true,
            black,
            surface_width,
            surface_height,
        );

        let fps_string = format!("FPS: {}", fps);
        draw::draw_string(
            frame,
            &fps_string,
            &font,
            white,
            100,
            100,
            surface_width,
            surface_height,
        );
    }
}

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();

    // load font
    let source = SystemSource::new();
    let handle = source
        .select_best_match(&[FamilyName::Monospace], &Properties::new())
        .unwrap();
    let font = handle.load().unwrap();

    let mut app = App {
        font: Some(font),
        ..Default::default()
    };

    event_loop.run_app(&mut app)
}
