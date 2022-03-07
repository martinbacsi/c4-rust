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
    pub fn add_sample(&mut self, s: Sample) {
        let s_2 = self.samples.get_mut(&s.hash);
        if s_2.is_some() {
            let mut s2 = s_2.unwrap();
            s2.visits += 1;
            for i in 0..POLICY_SIZE {
                s2.p[i] += s.p[i];
            }
            s2.v += s.v;
        } else {
            self.samples.insert(s.hash, s);
        }
    }
}
