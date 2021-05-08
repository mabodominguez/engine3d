use crate::game2_modes::*;
use crate::music::Sound;

use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::num::ParseIntError;

use std::fs::*;
use std::path::Path;
use std::rc::Rc;

pub fn load_game(gamefilepath: &str) -> Result<GameState, Box<dyn Error>> {
    // example path "./src/bin/assets.json" aka pathing starts from the top I think
    let json_file_path = Path::new(gamefilepath);
    // let json_file = File::open(json_file_path).expect("file not found");
    let json_file = File::open(json_file_path)?;

    let deserialized_state: SerDeGameState = serde_json::from_reader(json_file)?;

    let reconstituted_state = GameState {};
    return Ok(reconstituted_state);
}

pub fn save_game(state: &GameState, gamefilepath: &str) -> Result<bool, Box<dyn Error>> {
    let s = SerDeGameState {};
    let j = serde_json::to_string(&s)?;
    write(gamefilepath, j)?;
    return Ok(true);
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SerDeGameState {
    // alive: Vec<bool>,
// types: Vec<EntityType>,
// positions: Vec<Vec2i>,
// velocities: Vec<Vec2i>,
// sizes: Vec<(usize, usize)>,
// textures: Vec<TextureID>, // design decisions!
// anim_state: Vec<AnimationStateID>,
// inventory: ItemInventory,
// scroll: Vec2i, // here too!
// // Current level
// level: usize,
// mode: Mode,
// contacts: Vec<Contact>,
}
