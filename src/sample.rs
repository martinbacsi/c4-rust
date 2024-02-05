use std::collections::HashMap;

use crate::{node::Node};
use crate::game::*;
#[derive(Copy, Clone)]
pub struct Sample {
    pub input: [f32; STATE_SIZE],
    pub p: [f32; ACTION_SIZE],
    pub v: f32,
}

impl Sample {
    pub fn new(node: &Node, game: &Game, player: usize) -> Sample {
        Sample {
            input: game.encode(player),
            p: node.prob_vector(),
            v: node.q as f32 / node.visits as f32,
        }
    }
}
