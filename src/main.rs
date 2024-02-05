#![allow(dead_code)]
#![allow(unused_must_use)]

mod decode_base16k;
mod mcts;

mod nn;
mod nn_string;
mod node;
mod random;
mod sample;

mod xorshift;
mod game;
mod ugly;
mod entity;
mod scan;
mod vector;
mod fish;
mod drone;
mod collision;
mod player;
mod closest;


use crate::decode_base16k::encode_b16k;
use crate::nn::NN;
use mcts::MCTS;
use node::Node;
use rand::RngCore;
use std::collections::HashMap;
use std::env::args;
use std::fs;
use std::thread;

use std::{fs::File, io::Write};
struct Config {
    selfplay: bool,
    iters: usize,
    cpuct: f64,
    learning_rate: f64,
    load_file: bool
}

#[cfg(target_os = "linux")]
const CONF: Config = Config {
    selfplay: false,
    iters: usize::MAX,
    cpuct: 3.0,
};

#[cfg(target_os = "windows")]
const CONF: Config = Config {
    selfplay: true,
    iters: 100,
    cpuct: 4.0,
    learning_rate: 0.0001,
    load_file: true
};

pub const NNLEN: usize = 29322 * 4;

fn main() {
    if args().find(|a| a == "--encode").is_some() {
        let (_, enc) = encode_b16k("best.w32");
        let st = String::from_utf16(&enc).unwrap();

        let path = "src/nn_string.rs";
        fs::remove_file(path);
        let mut output = File::create(path).unwrap();
        output.write(b"pub const NNSTR: &str = \"");
        output.write(st.as_bytes());
        output.write(b"\";");
        output.flush();


    } else {
        #[cfg(target_os = "windows")]
        {
            loop {
                let mut handles = Vec::new();

                for _ in 0..1 {
                    let handle = thread::spawn(|| {
                        let mut ss = Vec::new();
                        let mut mcts = MCTS::new();
                        for i in 0..30 {
                            eprintln!("{}", i);
                            mcts.self_play(&mut ss);
                        }
                        
                        ss
                    });
                    handles.push(handle);
                }
                
                let mut combined_samples = Vec::new();
                for handle in handles {
                    combined_samples.extend(handle.join().unwrap());
                }

                let mut nn = NN::new();
                nn.train(&combined_samples);
            
            }
        }
        #[cfg(target_os = "linux")]
        MCTS::new().cg();
    }
}
