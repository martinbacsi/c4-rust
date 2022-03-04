mod connect4;
mod decode_base16k;
mod mcts;
mod nn;
mod nn_string;
mod node;
mod pool;
use crate::decode_base16k::decode_b16k;
use crate::decode_base16k::encode_b16k;
use crate::nn::NN;
use connect4::Connect4;
use connect4::Outcome;
use mcts::MCTS;
use nn::NNManager;
use node::Node;
use pool::Pool;
use std::time::{Duration, Instant};

struct config {
    selfplay: bool,
    iters: usize,
}

#[cfg(target_os = "linux")]
const conf: config = config {
    selfplay: false,
    iters: usize::MAX,
};

#[cfg(target_os = "windows")]
const conf: config = config {
    selfplay: true,
    iters: 2000,
};

const W: usize = 9;
const H: usize = 7;

const POLICY_SIZE: usize = W;
const INPUT_SIZE: usize = H * W * 2;

const cpuct: f64 = 4.0;

fn main() {
    let mut mcts = MCTS::new();
    #[cfg(target_os = "windows")]
    mcts.play_against();
    #[cfg(target_os = "linux")]
    mcts.cg();
}
