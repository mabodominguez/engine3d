use crate::music::*;
use crate::saveload::*;
use rodio::{Sink, Source};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Mode {
    Title,
    Play(PlayMode),
    EndGame,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PlayMode {
    Menu,
    Battle,
    Inventory,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ItemInventory {
    pub bridge: i32,
    pub shovel: i32,
    pub ladder: i32,
}

#[derive(Clone)]
pub struct GameState {}

impl Mode {
    pub fn update(&self) {
        match self {
            Mode::Title => {}
            Mode::Play(pm) => {}

            Mode::EndGame => {}
        }
    }

    pub fn display(&self) {
        match self {
            Mode::Title => {}
            Mode::Play(pm) => {}
            Mode::EndGame => {}
        }
    }
}

impl PlayMode {
    fn update(&self) {
        match self {
            PlayMode::Menu => {}
            PlayMode::Inventory => {}
            PlayMode::Battle => {}
        }
    }
    fn display(&self) {
        match self {
            PlayMode::Menu => {}
            PlayMode::Inventory => {}

            PlayMode::Battle => {}
        }
    }
}
