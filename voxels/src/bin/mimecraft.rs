use engine3d::assets::{Asset2d, Assets, Object2d};
use engine3d::camera::*;
use engine3d::events::*;
use engine3d::model::*;
use engine3d::render::TwoDID;
use engine3d::voxel::{Chunk, VOXEL_HALFWIDTH, CHUNK_SIZE};
use engine3d::player::Player;
use engine3d::geom::{BBox, dist_3d};
use engine3d::{Engine, Game};
use engine3d::collision::*;
use engine3d::coordinates::*;
use engine3d::particle::Particle;
use engine3d::world_gen::{WORLD_MAX};
pub type Pos3 = cgmath::Point3<f32>;
pub type Pos2 = cgmath::Point2<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;
pub use winit::event::VirtualKeyCode as KeyCode;

pub struct Game1 {
    camera_pos: Pos3,
    chunks: Vec<Chunk>,
    twods: Vec<TwoDID>,
    rule: Rule,
    player: Player,
    contacts: Contacts,
    particles: Vec<Particle>,
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
                x: 10.0,
                y: 90.0,
                z: 90.0,
            },
            chunks: vec![],
            rule: Rule::Title,
            twods: vec![],
            player: Player::new(BBox{center:Pos3{x:10.0, y:350.0, z:10.0}, halfwidth:VOXEL_HALFWIDTH}),
            contacts: Contacts::new(),
            particles: vec![],
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
            /*engine3d::assets::Asset2d(
                std::path::Path::new(env!("OUT_DIR"))
                    .join("content")
                    .join("hotbar_highlight.png"),
                "hotbar_highlight".to_string(),
            ),*/
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
                self.player.process_events(&engine.events);
                engine.render.input(&engine.events, *i);
                //collision option 1
                let (mut chunk_index, _) = world_to_chunk(self.player.hitbox.center);
                if engine.render.chunks.len() == 0 {
                    chunk_index = 0;
                } else {
                    chunk_index = chunk_index.clamp(0, engine.render.chunks.len()-1);
                    let chunk = &mut engine.render.chunks[chunk_index];
                    if !chunk.bboxes_generated(){chunk.create_bboxes(chunk_index);println!("generated {}", chunk_index);}
                    update(chunk, &mut self.player.hitbox, &mut self.particles, &mut self.contacts);
                    self.player.process_contacts(&self.contacts.block_player);
                }
                //collision option 2
                // for c in 0..engine.render.chunks.len() {
                //     let chunk_pos = index_to_world(c);
                //     let chunk_posf = Pos3{x:chunk_pos.0 as f32, y:chunk_pos.1 as f32, z:chunk_pos.2 as f32};
                //     if dist_3d(self.player.hitbox.center, chunk_posf) <= (CHUNK_SIZE * 6) as f32 * VOXEL_HALFWIDTH {
                //         let chunk = &mut engine.render.chunks[c];
                //         if !chunk.bboxes_generated(){chunk.create_bboxes(c);println!("generated {}", c);}
                //         update(chunk, &mut self.player.hitbox, &mut self.particles, &mut self.contacts);
                //         self.player.process_contacts(&self.contacts.block_player);
                //     }
                // }
                self.player.update(&mut engine.render.camera);
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
                if engine.events.key_pressed(KeyCode::T) {
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
