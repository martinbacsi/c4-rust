#![allow(dead_code)]
#![allow(unused_must_use)]

mod connect4;
mod decode_base16k;
mod mcts;
mod nn;
mod nn_string;
mod node;
mod pool;
mod random;
mod sample;
use crate::decode_base16k::encode_b16k;
use connect4::Connect4;
use connect4::Outcome;
use mcts::MCTS;
use nn::NNManager;
use node::Node;
use pool::Pool;
use sample::SampleStore;
use std::collections::HashMap;
use std::env::args;
use std::fs;
use std::thread;

use std::{fs::File, io::Write};
struct Config {
    selfplay: bool,
    iters: usize,
}

#[cfg(target_os = "linux")]
const CONF: config = Config {
    selfplay: false,
    iters: usize::MAX,
};

#[cfg(target_os = "windows")]
const CONF: Config = Config {
    selfplay: true,
    iters: 10000,
};

const W: usize = 9;
const H: usize = 7;

const POLICY_SIZE: usize = W;
const INPUT_SIZE: usize = H * W * 2;

const CPUCT: f64 = 3.0;

pub const NNLEN: usize = 33482 * 4;

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
            let mut handles = Vec::new();

            for _ in 0..4 {
                let handle = thread::spawn(|| {
                    let mut ss: SampleStore = SampleStore {
                        samples: HashMap::new(),
                    };
                    let mut mcts = MCTS::new();
                    for i in 0..250 {
                        eprintln!("{}", i);
                        mcts.self_play(&mut ss);
                        mcts.clear();
                    }
                    let mut file =
                        File::create(String::from("./traindata/") + &random::rand().to_string())
                            .unwrap();
                    for (_, s) in &mut ss.samples {
                        s.v /= s.visits as f32;
                        for p in s.p.iter_mut() {
                            *p /= s.visits as f32;
                        }
                        let a: [u8; INPUT_SIZE * 4] = unsafe { std::mem::transmute(s.input) };
                        file.write_all(&a);
                        let a: [u8; POLICY_SIZE * 4] = unsafe { std::mem::transmute(s.p) };
                        file.write_all(&a);
                        let a: [u8; 1 * 4] = unsafe { std::mem::transmute(s.v) };
                        file.write_all(&a);
                    }
                });
                handles.push(handle);
            }
            for h in handles.into_iter() {
                h.join();
            }
        }
        #[cfg(target_os = "linux")]
        MCTS::new().cg();
    }
}
