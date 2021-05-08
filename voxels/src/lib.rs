use std::path::Path;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

pub mod camera;
pub mod camera_control;
// pub mod collision;
pub mod events;
pub mod gamestate;
pub mod geom;
pub mod model;
pub mod texture;
pub mod voxel;
pub mod collision;
pub mod particle;
pub mod instance_raw;
use events::Events;
pub mod render;
use render::Render;
pub mod assets;
use assets::Assets;

pub const DT: f32 = 1.0 / 60.0;

pub trait Game: Sized {
    type StaticData;
    fn start(engine: &mut Engine) -> (Self, Self::StaticData);
    fn update(&mut self, rules: &Self::StaticData, engine: &mut Engine);
    fn render(&mut self, rules: &Self::StaticData, assets: &Assets);
}

pub struct Engine {
    pub frame: usize,
    pub assets: Assets,
    pub render: Render,
    pub events: Events,
}

// pub enum Rule {
//     Title,
//     Play,
//     End,
// }

impl Engine {
    // do we need this? i don't think so
    pub fn load_model(&mut self, model: impl AsRef<Path>) -> assets::ModelRef {
        self.assets.load_model(
            &self.render.device,
            &self.render.queue,
            &self.render.texture_layout,
            model,
        )
    }
    // pub fn load_gltf(
    //     &mut self,
    //     gltf: impl AsRef<Path>,
    // ) -> (
    //     Vec<assets::ModelRef>,
    //     Vec<assets::RigRef>,
    //     Vec<assets::AnimRef>,
    // ) {
    //     self.assets.load_gltf(
    //         &self.render.device,
    //         &self.render.queue,
    //         &self.render.texture_layout,
    //         gltf,
    //     )
    // }
    // pub fn camera_mut(&mut self) -> &mut camera::Camera {
    //     &mut self.render.camera
    // }
}

pub fn run<R, G: Game<StaticData = R>>(
    window_builder: winit::window::WindowBuilder,
    asset_root: &Path,
) {
    use std::time::Instant;
    let mut event_loop = EventLoop::new();
    let window = window_builder.build(&event_loop).unwrap();
    let _window_grab = window.set_cursor_grab(true);
    window.set_cursor_icon(winit::window::CursorIcon::Crosshair);
    let assets = Assets::new(asset_root);
    use futures::executor::block_on;
    let render = block_on(Render::new(&window));
    let events = Events::default();
    let mut engine = Engine {
        assets,
        render,
        events,
        frame: 0,
    };
    let (mut game, rules) = G::start(&mut engine);
    let mut available_time: f32 = 0.0;
    let mut since = Instant::now();
    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent { ref event, .. } => engine.events.device_event(event),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                engine.events.window_event(event);

                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        engine.render.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        engine.render.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                match engine.render.render(&mut game, &rules, &mut engine.assets) {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => engine.render.resize(engine.render.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
                
                // The renderer "produces" time...
                available_time += since.elapsed().as_secs_f32();
                since = Instant::now();
            }
            _ => {}
        }
        // And the simulation "consumes" it
        while available_time >= DT {
            // Eat up one frame worth of time
            available_time -= DT;
            game.update(&rules, &mut engine);
            let _window_set_cursor = window.set_cursor_position(winit::dpi::PhysicalPosition::new(engine.render.camera_controller.center_x, engine.render.camera_controller.center_y));
            engine.events.next_frame();
            engine.frame += 1;
            // Increment the frame counter
            //frame_count += 1;
        }
    });
}
