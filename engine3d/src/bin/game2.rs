use std::path::Path;
use std::rc::Rc;

use rodio::Sink;
use winit::dpi::LogicalSize;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 128;
const HEIGHT: usize = 128;

fn main() {
    // audio initialization
    // let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    // let sink = Sound::sink(&stream_handle);

    let window_builder = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Game3D")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
    };
}

// fn draw_game(screen: &mut Screen) {
//     state.mode.display(screen);
// }

// fn update_game() {
//     state.mode.update();
// }
