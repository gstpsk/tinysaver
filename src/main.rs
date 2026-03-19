use font_kit::{family_name::FamilyName, font::Font, properties::Properties, source::SystemSource};
use pixels::{Pixels, SurfaceTexture, wgpu::Backend};
use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    monitor::MonitorHandle,
    window::{Fullscreen, Window, WindowBuilder},
};

use crate::dvd::DvdState;

mod utils;
mod draw;
mod color;
mod dvd;
mod test;


fn backend_to_str(backend: Backend) -> &'static str { // return static string slice
    match backend {
        Backend::Vulkan => "Vulkan",
        Backend::Metal => "Metal",
        Backend::Dx12 => "DirectX 12",
        Backend::Dx11 => "DirectX 11",
        Backend::Gl => "OpenGL",
        Backend::BrowserWebGpu => "WebGPU",
        _ => "Unknown",
    }
}

fn main() {
    let event_loop = EventLoop::new();



    // select ANY system font
    let source = SystemSource::new();

    let handle = source
        .select_best_match(&[FamilyName::Monospace], &Properties::new())
        .unwrap();

    let font = handle.load().unwrap();

    let mut frame_count: u32 = 0;

    let mut window: Option<Window> = None;
    let mut pixels: Option<Pixels> = None;
    let mut dvd_state: Option<DvdState> = None;

    let mut fps: u32 = 0;    

    event_loop.run(move |event, event_loop_window_target, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(_) => {
                    // pixels
                    //     .resize_surface(size.width, size.height)
                    //     .expect("resized surface");
                }
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode.unwrap() {
                    VirtualKeyCode::Q => {
                        std::process::exit(0);
                    }
                    VirtualKeyCode::Up => {
                        if let Some(state) = &mut dvd_state {
                            state.increase_speed_by(1);
                        } else {
                            eprintln!("Can't increase speed of None!");
                        }
                    }
                    VirtualKeyCode::Down => {
                        if let Some(state) = &mut dvd_state {
                            state.decrease_speed_by(1);
                        } else {
                            eprintln!("Can't decrease speed of None!");
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Event::Resumed => {
                    let new_window = WindowBuilder::new()
                        .with_fullscreen(Some(Fullscreen::Borderless(None)))
                        .build(&event_loop_window_target)
                        .unwrap();

                    // set inner size to primary monitor size
                    //let size = new_window.current_monitor().unwrap().size(); // doesnt work
                    //let size = new_window.primary_monitor().unwrap().size(); // doesnt work
                    let monitors: Vec<MonitorHandle> = new_window.available_monitors().collect();
                    let size = monitors.first().unwrap().size(); // this works but is terrible

                    new_window.set_inner_size(size);
                    
                    // Create surface
                    let surface = SurfaceTexture::new(size.width, size.height, &new_window);

                    // Create pixels
                    let new_pixels = Pixels::new(size.width, size.height, surface).unwrap();

                    println!("GPU: {}", new_pixels.adapter().get_info().name);
                    println!("Backend: {}", backend_to_str(new_pixels.adapter().get_info().backend));

                    window = Some(new_window);
                    pixels = Some(new_pixels);
                    
                    dvd_state = Some(DvdState::with_random_position(1, size.width as i32, size.height as i32));
            },
            Event::RedrawRequested(_) => {
                // if window, pixels  dvd_state are not None we can use them
                if let (Some(window), Some(pixels), Some(dvd_state)) = (&window, &mut pixels, &mut dvd_state) {
                    // save start time for fps computation
                    let time_start = std::time::Instant::now();
                    
                    let frame = pixels.frame_mut();

                    dvd::dvd_style(frame, frame_count, dvd_state);
                    draw_fps(frame, frame_count, fps, &font, window.inner_size().width as i32, window.inner_size().height as i32);
                                        
                    pixels.render().unwrap();
                    frame_count += 1;

                    // compute fps
                    fps = (std::time::Duration::from_secs(1).as_nanos() as f64 / time_start.elapsed().as_nanos() as f64) as u32;
                }

            }

            Event::MainEventsCleared => {
                if let Some(window) = &window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    });
}


fn draw_fps(frame: &mut [u8], frame_count: u32, fps: u32, font: &Font, surface_width: i32, surface_height: i32) {
    let black = (0,0,0,0);
    let white = (255,255,255);
    if frame_count % 30 == 0 {    
        draw::draw_rect(frame, 100, 100, 500, 200, 0, None, true, black, surface_width, surface_height);
        
        let fps_string = format!("FPS: {}", fps);
        draw::draw_string(frame, &fps_string, &font, white, 100, 100, surface_width, surface_height);
    }
}