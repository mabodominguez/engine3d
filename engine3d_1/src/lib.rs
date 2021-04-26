use pixels::{Pixels, SurfaceTexture};
use rodio::Sink;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
pub mod game2_modes;
pub mod music;
pub mod saveload;

const DEPTH: usize = 4;
const DT: f64 = 1.0 / 60.0;

pub fn run<Rule, State>(
    width: usize,
    height: usize,
    window_builder: WindowBuilder,
    draw: impl Fn() + 'static,
    update: impl Fn() + 'static,
) {
    use std::time::Instant;

    let mut event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = window_builder.build(&event_loop).unwrap();
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(width as u32, height as u32, surface_texture).unwrap()
    };

    // How many frames have we simulated?
    let mut frame_count: usize = 0;
    // How many unsimulated frames have we saved up?
    let mut available_time = 0.0;
    let mut since = Instant::now();
    event_loop.run_return(|event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            // Flip buffers
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Rendering has used up some time.
            // The renderer "produces" time...
            available_time += since.elapsed().as_secs_f64();
        }
        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            // Resize the window if needed
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }
        }
        // And the simulation "consumes" it
        while available_time >= DT {
            // Eat up one frame worth of time
            available_time -= DT;

            update();

            // Increment the frame counter
            frame_count += 1;
        }
        // Request redraw
        window.request_redraw();
        // When did the last frame end?
        since = Instant::now();
    });
}
