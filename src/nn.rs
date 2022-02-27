use std::{collections::HashMap};
use crate::Connect4;
use crate::POLICY_SIZE;
pub struct NnOutput {
    pub p: [f32; POLICY_SIZE],
    pub v: f32
}
struct NN {

}

pub struct NNManager {
    pub cache: HashMap<usize, NnOutput>
}

impl NNManager {
    pub fn Get(&mut self, game: &Connect4) -> &NnOutput {
        let hash = game.hash();
        if !self.cache.contains_key(&hash) {
            self.cache.insert(hash,NnOutput{p: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0., 0.0, 0.0], v: 0.0});
        }
        &self.cache[&hash]
    }
}