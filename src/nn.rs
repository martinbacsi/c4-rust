use std::{collections::HashMap};
use crate::Connect4;
use crate::POLICY_SIZE;
pub struct NnOutput {
    pub p: [f32; POLICY_SIZE],
    pub v: f32
}

struct DenseLayer {
    input: Vec<f32>,
    weights: Vec<f32>,
    bias: Vec<f32>
}

impl DenseLayer {
    fn forward(&self, output: &mut Vec<f32>) {
        output.copy_from_slice(&self.bias);
        let out_size = output.len();
        for j in 0..self.input.len() {
            let val = self.input[j];
            if val != 0.0 {
                for i in 0..out_size {
                    output[i] += val * self.weights[j * out_size + i];
                }
            } 
        }
    }

    fn forward_game(&self, game: &Connect4, output: &mut Vec<f32>) {
        output.copy_from_slice(&self.bias);
        let out_size = output.len();
        let mut maps = [game.my_bb, game.op_bb];
        for i in 0..2 {
            while maps[i] != 0 {
                let r = maps[i].trailing_zeros();
                maps[i] ^= 1 << r;
                let nn_ind  = r as usize * 2  + i;
                for j in 0 ..out_size {
                    output[i] +=  1. * self.weights[j + out_size * nn_ind];
                }
            }           
        }
    }
}

struct NN {
    path: Vec<DenseLayer>
}

pub struct NNManager {
    pub cache: HashMap<usize, NnOutput>
}

impl NNManager {
    pub fn Get(&mut self, game: &Connect4) -> &NnOutput {
        let hash = game.hash();
        if !self.cache.contains_key(&hash) {
            self.cache.insert(hash,NnOutput{p: [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], v: 0.0});
        }
        &self.cache[&hash]
    }
}