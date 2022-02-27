use std::{collections::HashMap};
use crate::Connect4;
use std::fs::File;
use std::io::{BufReader, Read};
use crate::POLICY_SIZE;
use crate::INPUT_SIZE;
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

    pub fn new(input_size: usize) -> DenseLayer {
        let mut r = DenseLayer{
            input: Vec::new(),
            weights: Vec::new(),
            bias: Vec::new()
        };
        r.input.resize(input_size, 0.);
        r
    }
}

pub struct NN {
    path: Vec<DenseLayer>
}

fn relu(v: &mut Vec<f32>) {
    for i in v.iter_mut() {
        *i = i.max(0.);
    }
}

fn softmax(v: &mut [f32; POLICY_SIZE]) {
    let max = v.iter().fold(f32::NEG_INFINITY, |max, i| {if *i > max {*i} else {max}});
    let mut sum: f32 = 0.;
    v.iter_mut().for_each(|i| {
        *i = f32::exp(*i - max); 
        sum += *i;
    });

    v.iter_mut().for_each(|i| (*i) /= sum );
}

impl NN {
    pub fn forward(&mut self, game: &Connect4) -> NnOutput {
        //todo dont reinit vec
        let mut res_raw = vec![0.; POLICY_SIZE + 1];
        //let mut res_raw: [f32; POLICY_SIZE + 1] = [1.; POLICY_SIZE + 1];
        let mut res = NnOutput{p: [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], v: 0.0};

        for i in 0.. self.path.len() - 1 {
            let (a, b) = self.path.split_at_mut(i + 1);
            if i == 0 {
                a.last().unwrap().forward_game(&game,  &mut b[0].input);
            } else {
                a.last().unwrap().forward(&mut b[0].input);
            }
            relu(&mut b[0].input);
        }

        self.path.last().unwrap().forward(&mut res_raw);
        
        for i in 0..POLICY_SIZE {
           res.p[i] = res_raw[i]; 
        }
        res.v = f32::tanh(res_raw[POLICY_SIZE]);
        softmax(&mut res.p);
        res
    }

    pub fn new() -> NN {
        NN{
            path: vec![
                DenseLayer::new(INPUT_SIZE),
                DenseLayer::new(128),
                DenseLayer::new(64),
            ]
        }
    }

    pub fn read_weights(&mut self, file_name: &str) {
        let mut input = BufReader::new(
            File::open(file_name).expect("kakus")
        );

        let mut buffer = [0; std::mem::size_of::<f32>()];
        let mut all_weights = Vec::new();
        
        loop {
            use std::io::ErrorKind;
            let res = input.read_exact(&mut buffer);
            match res {
                Err(error) if error.kind() == ErrorKind::UnexpectedEof => break,
                _ => {}
            }
            res.expect("error while reading");
            let f = f32::from_le_bytes(buffer);
            all_weights.push(f);
        }

        let mut id = 0;
        for i in 0..self.path.len() {
            let next_size = if i == self.path.len() - 1 { POLICY_SIZE + 1} else {self.path[i + 1].input.len()};
            let weights_size = self.path[i].input.len() * next_size;
            self.path[i].weights.resize(weights_size, 0.0);
            self.path[i].bias.resize(next_size, 0.0);
            for j in 0..weights_size {
                self.path[i].weights[j] = all_weights[id];
                id+=1;
            }

            for j in 0..next_size {
                self.path[i].bias[j] = all_weights[id];
                id+=1;
            }
        }
    }
}

pub struct NNManager {
    pub cache: HashMap<usize, NnOutput>,
    pub nn: NN
}

impl NNManager {
    pub fn Get(&mut self, game: &Connect4) -> &NnOutput {
        let hash = game.hash();
        if !self.cache.contains_key(&hash) {
            self.cache.insert(hash,self.nn.forward(game));
        }
        &self.cache[&hash]
    }
}