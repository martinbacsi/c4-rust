use std::collections::HashMap;

use crate::{connect4::Connect4, node::Node, INPUT_SIZE, POLICY_SIZE};

#[derive(Copy, Clone)]
pub struct Sample {
    pub input: [f32; INPUT_SIZE],
    pub p: [f32; POLICY_SIZE],
    pub v: f32,
    pub visits: usize,
    pub hash: usize,
}

impl Sample {
    pub fn new(node: &Node) -> Sample {
        let mut sample: Sample = Sample {
            input: [0.0; INPUT_SIZE],
            p: node.prob_vector(),
            v: node.Q as f32 / node.visits as f32,
            visits: 1,
            hash: node.game.hash(),
        };

        node.game.on_set_indices(|i| sample.input[i] = 1.);
        sample
    }
}

pub struct SampleStore {
    pub samples: HashMap<usize, Sample>,
}

impl SampleStore {
    pub fn add_sample(&mut self, mut s: Sample) {
        if self.samples.contains_key(&s.hash) {
            let s2 = &self.samples[&s.hash];
            s.visits += s2.visits;
            s.input.clone_from_slice(&s2.input);
            for i in 0..POLICY_SIZE {
                s.p[i] += s2.p[i];
            }
            s.v += s2.v;
        }

        self.samples.insert(s.hash, s);
    }
}
