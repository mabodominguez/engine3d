use engine3d::assets::*;
use engine3d::camera::*;
use engine3d::events::*;
use engine3d::model::*;
use engine3d::voxel::Chunk;
use engine3d::{Engine, Game};
pub type Pos3 = cgmath::Point3<f32>;
pub type Pos2 = cgmath::Point2<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

pub struct Game1 {
    camera_pos: Pos3,
    chunks: Vec<Chunk>,
}
pub enum Rule {
    Title,
    Play,
    End,
}
impl Game for Game1 {
    type StaticData = Rule;
    fn start(engine: &mut Engine) -> (Self, Self::StaticData) {
        let game = Game1 {
            camera_pos: Pos3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            chunks: vec![],
        };
        return (game, Rule::Play);
    }
    fn update(&mut self, rules: &Self::StaticData, engine: &mut Engine) {
        match rules {
            Rule::Title => {
                // if engine.events.key_pressed(Keycode::Space) {
                //     // Somehow change to the play screen
                // }
                // here is a good place to handle loading
                // render title screen
                // handle key press to go to play
            }
            Rule::Play => {
                // Change this with new camera code and stuff
                // engine.camera_controller.update_camera(&mut engine.render.camera);
                // render hot bar and stuff? change assets unclear
                engine.render.input(&engine.events);
                engine.render.update();
            }
            Rule::End => {
                // render end screen
                // handle key press to panic
            }
        }
    }
    fn render(&mut self, rules: &Self::StaticData, assets: &Assets) {
        match rules {
            Rule::Title => {
            }
            Rule::Play => {
            }
            Rule::End => {
            }
        }
    }
}

fn main() {
    let mut game = Game1 {
        camera_pos: Pos3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        chunks: vec![],
    };
    let title = "mimecraft";
    let asset_root = std::path::Path::new(env!("OUT_DIR")).join("content");
    let window_builder = winit::window::WindowBuilder::new().with_title(title);
    engine3d::run::<Rule, Game1>(window_builder, &asset_root);
}
