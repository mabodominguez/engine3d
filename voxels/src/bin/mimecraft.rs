use engine3d::assets::{Asset2d, Assets, Object2d};
use engine3d::camera::*;
use engine3d::events::*;
use engine3d::model::*;
use engine3d::voxel::Chunk;
use engine3d::{Engine, Game};
pub type Pos3 = cgmath::Point3<f32>;
pub type Pos2 = cgmath::Point2<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;
pub use winit::event::VirtualKeyCode as KeyCode;

pub struct Game1 {
    camera_pos: Pos3,
    chunks: Vec<Chunk>,
}
pub enum Rule {
    Title,
    Play(usize), // what part of the inventory
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
                // potentially move this to start? unsure
                let title_bg = vec![engine3d::assets::Asset2d(
                    std::path::Path::new(env!("OUT_DIR"))
                        .join("content")
                        .join("titlescreen.png"),
                )];
                let title_objects = vec![engine3d::assets::Object2d {
                    bg: 0,
                    verts: [
                        VertexTwoD {
                            position: [-0.9, -0.5], // make 0s -1s (x and y go from -1 to 1)
                            tex_coords: [0.0, 0.0],
                        },
                        VertexTwoD {
                            position: [-0.9, -0.9],
                            tex_coords: [0.0, 1.0],
                        },
                        VertexTwoD {
                            position: [0.9, -0.5],
                            tex_coords: [1.0, 0.0],
                        },
                        VertexTwoD {
                            position: [0.9, -0.9],
                            tex_coords: [1.0, 1.0],
                        },
                    ],
                    visible: true,
                }];
                engine.render.set_2d_bind_groups(&title_bg);
                engine.render.set_2d_buffers(&title_objects);

                if engine.events.key_pressed(KeyCode::Space) {
                    // change rule to play
                    // Somehow change to the play screen
                }
                // here is a good place to handle loading
                // render title screen
                // handle key press to go to play
            }
            Rule::Play(i) => {
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
