use engine3d::assets::{Asset2d, Assets, Object2d};
use engine3d::camera::*;
use engine3d::events::*;
use engine3d::model::*;
use engine3d::render::TwoDID;
use engine3d::voxel::Chunk;
use engine3d::{Engine, Game};
pub type Pos3 = cgmath::Point3<f32>;
pub type Pos2 = cgmath::Point2<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;
pub use winit::event::VirtualKeyCode as KeyCode;

pub struct Game1 {
    camera_pos: Pos3,
    chunks: Vec<Chunk>,
    twods: Vec<TwoDID>,
    rule: Rule,
}
#[derive(Debug)]
pub enum Rule {
    Title,
    Play(u8), // what part of the inventory
    End,
}
impl Game for Game1 {
    type StaticData = Rule;
    fn start(engine: &mut Engine) -> (Self, Self::StaticData) {
        let mut game = Game1 {
            camera_pos: Pos3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            chunks: vec![],
            rule: Rule::Title,
            twods: vec![],
        };
        let bind_groups = vec![
            engine3d::assets::Asset2d(
                std::path::Path::new(env!("OUT_DIR"))
                    .join("content")
                    .join("titlescreen.png"),
                "title".to_string(),
            ),
            engine3d::assets::Asset2d(
                std::path::Path::new(env!("OUT_DIR"))
                    .join("content")
                    .join("hotbar.png"),
                "hotbar".to_string(),
            ),
            engine3d::assets::Asset2d(
                std::path::Path::new(env!("OUT_DIR"))
                    .join("content")
                    .join("hotbar_highlight.png"),
                "hotbar_highlight".to_string(),
            ),
            engine3d::assets::Asset2d(
                std::path::Path::new(env!("OUT_DIR"))
                    .join("content")
                    .join("endscreen.png"),
                "endscreen".to_string(),
            ),
        ];
        let objects_2d = vec![
            // title screen
            engine3d::assets::Object2d {
                bg: 0,
                verts: [
                    VertexTwoD {
                        position: [-1.0, 1.0], // make 0s -1s (x and y go from -1 to 1)
                        tex_coords: [0.0, 0.0],
                    },
                    VertexTwoD {
                        position: [-1.0, -1.0],
                        tex_coords: [0.0, 1.0],
                    },
                    VertexTwoD {
                        position: [1.0, 1.0],
                        tex_coords: [1.0, 0.0],
                    },
                    VertexTwoD {
                        position: [1.0, -1.0],
                        tex_coords: [1.0, 1.0],
                    },
                ],
                visible: true,
            }, // hot bar
            engine3d::assets::Object2d {
                bg: 1,
                verts: [
                    VertexTwoD {
                        position: [-0.9, -0.5],
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
                visible: false,
            }, // hotbar highlight
            engine3d::assets::Object2d {
                bg: 2,
                verts: [
                    VertexTwoD {
                        position: [-0.9, -0.5],
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
                visible: false,
            }, // end screen
            engine3d::assets::Object2d {
                bg: 3,
                verts: [
                    VertexTwoD {
                        position: [-1.0, 1.0], // make 0s -1s (x and y go from -1 to 1)
                        tex_coords: [0.0, 0.0],
                    },
                    VertexTwoD {
                        position: [-1.0, -1.0],
                        tex_coords: [0.0, 1.0],
                    },
                    VertexTwoD {
                        position: [1.0, 1.0],
                        tex_coords: [1.0, 0.0],
                    },
                    VertexTwoD {
                        position: [1.0, -1.0],
                        tex_coords: [1.0, 1.0],
                    },
                ],
                visible: false,
            },
        ];
        engine.render.set_2d_bind_groups(&bind_groups);
        game.twods = engine.render.set_2d_buffers(&objects_2d);
        return (game, Rule::Title);
    }
    fn update(&mut self, rules: &mut Self::StaticData, engine: &mut Engine) {
        // println!("{:?}", rules);
        match rules {
            Rule::Title => {
                if engine.events.key_pressed(KeyCode::L) {
                    // TODO: insert load game here
                    engine.render.chunks = engine3d::save::load();
                    // start gameplay
                    *rules = Rule::Play(1);
                    engine.render.objects_2d[0].2 = false;
                }
                if engine.events.key_pressed(KeyCode::Space) {
                    *rules = Rule::Play(1);
                    engine.render.objects_2d[0].2 = false;
                    // change rule to play
                }
            }
            Rule::Play(i) => {
                engine.render.input(&engine.events, *i);
                engine.render.update();
                // Change this with new camera code and stuff
                // render gameplay relevent stuff

                // render hotbar + hotbar highlight
                engine.render.objects_2d[1].2 = true;
                // engine.render.objects_2d[2].2 = true;
                let mut moved_highlight = Object2d {
                    bg: 2,
                    verts: [
                        VertexTwoD {
                            position: [-0.855, -0.551],
                            tex_coords: [0.0, 0.0],
                        },
                        VertexTwoD {
                            position: [-0.586, -0.551],
                            tex_coords: [0.0, 1.0],
                        },
                        VertexTwoD {
                            position: [-0.855, -0.827],
                            tex_coords: [1.0, 0.0],
                        },
                        VertexTwoD {
                            position: [-0.586, -0.827],
                            tex_coords: [1.0, 1.0],
                        },
                    ],
                    visible: false,
                };
                if engine.events.key_pressed(KeyCode::Q) {
                    *rules = Rule::End;
                    engine.render.objects_2d[1].2 = false;
                    engine.render.objects_2d[2].2 = false;
                }

                if engine.events.key_pressed(KeyCode::Key1) {
                    *rules = Rule::Play(1);
                    // moved_highlight.verts[]
                    engine
                        .render
                        .update_2d_buffer(&moved_highlight, engine.render.objects_2d[2]);
                    engine.render.objects_2d[2].2 = true;
                } else if engine.events.key_pressed(KeyCode::Key2) {
                    *rules = Rule::Play(2);
                    moved_highlight.verts = [
                        VertexTwoD {
                            position: [-0.563, -0.551],
                            tex_coords: [0.0, 0.0],
                        },
                        VertexTwoD {
                            position: [-0.293, -0.551],
                            tex_coords: [0.0, 1.0],
                        },
                        VertexTwoD {
                            position: [-0.563, -0.84],
                            tex_coords: [1.0, 0.0],
                        },
                        VertexTwoD {
                            position: [-0.293, -0.84],
                            tex_coords: [1.0, 1.0],
                        },
                    ];
                    engine
                        .render
                        .update_2d_buffer(&moved_highlight, engine.render.objects_2d[2]);
                    engine.render.objects_2d[2].2 = true;
                } else if engine.events.key_pressed(KeyCode::Key3) {
                    *rules = Rule::Play(3);
                    moved_highlight.verts = [
                        VertexTwoD {
                            position: [-0.273, -0.551],
                            tex_coords: [0.0, 0.0],
                        },
                        VertexTwoD {
                            position: [-0.004, -0.551],
                            tex_coords: [0.0, 1.0],
                        },
                        VertexTwoD {
                            position: [-0.273, -0.84],
                            tex_coords: [1.0, 0.0],
                        },
                        VertexTwoD {
                            position: [-0.004, -0.84],
                            tex_coords: [1.0, 1.0],
                        },
                    ];
                    engine
                        .render
                        .update_2d_buffer(&moved_highlight, engine.render.objects_2d[2]);
                    engine.render.objects_2d[2].2 = true;
                } else if engine.events.key_pressed(KeyCode::Key4) {
                    *rules = Rule::Play(4);
                    moved_highlight.verts = [
                        VertexTwoD {
                            position: [0.016, -0.551],
                            tex_coords: [0.0, 0.0],
                        },
                        VertexTwoD {
                            position: [0.286, -0.551],
                            tex_coords: [0.0, 1.0],
                        },
                        VertexTwoD {
                            position: [0.016, -0.84],
                            tex_coords: [1.0, 0.0],
                        },
                        VertexTwoD {
                            position: [0.286, -0.84],
                            tex_coords: [1.0, 1.0],
                        },
                    ];
                    engine
                        .render
                        .update_2d_buffer(&moved_highlight, engine.render.objects_2d[2]);
                    engine.render.objects_2d[2].2 = true;
                } else if engine.events.key_pressed(KeyCode::Key5) {
                    *rules = Rule::Play(5);
                    moved_highlight.verts = [
                        VertexTwoD {
                            position: [0.308, -0.551],
                            tex_coords: [0.0, 0.0],
                        },
                        VertexTwoD {
                            position: [0.578, -0.551],
                            tex_coords: [0.0, 1.0],
                        },
                        VertexTwoD {
                            position: [0.308, -0.84],
                            tex_coords: [1.0, 0.0],
                        },
                        VertexTwoD {
                            position: [0.578, -0.84],
                            tex_coords: [1.0, 1.0],
                        },
                    ];
                    engine
                        .render
                        .update_2d_buffer(&moved_highlight, engine.render.objects_2d[2]);
                    engine.render.objects_2d[2].2 = true;
                }
                if engine.events.key_pressed(KeyCode::S) {
                    engine3d::save::save(&engine.render.chunks);
                }

                // here is a good place to handle loading
                // render title screen
                // handle key press to go to play
                // Change this with new camera code and stuff
                // engine.camera_controller.update_camera(&mut engine.render.camera);
                // render hot bar and stuff? change assets unclear
            }
            Rule::End => {
                engine.render.objects_2d[3].2 = true;
                if engine.events.key_pressed(KeyCode::Q) {
                    panic!();
                }
                if engine.events.key_pressed(KeyCode::S) {
                    engine3d::save::save(&engine.render.chunks);
                }
            }
        }
    }
    fn render(&mut self, rules: &Self::StaticData, assets: &Assets) {}
}

fn main() {
    let title = "mimecraft";
    let asset_root = std::path::Path::new(env!("OUT_DIR")).join("content");
    let window_builder = winit::window::WindowBuilder::new().with_title(title);
    engine3d::run::<Rule, Game1>(window_builder, &asset_root);
}
