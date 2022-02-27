mod connect4;
mod nn;
mod node;
mod mcts;
mod pool;
mod random;
use nn::NNManager;
use node::Node;
use connect4::Connect4;
use connect4::Outcome;
use std::time::{Instant, Duration};
use mcts::MCTS;
use pool::Pool;
use crate::random::dirichlet_noise;


struct config {
    selfplay: bool,
    iters: usize
}

#[cfg(target_os = "linux")]
const conf: config = config {
    selfplay: false,
    iters: usize::MAX
};

#[cfg(target_os = "windows")]
const conf: config = config {
    selfplay: true,
    iters: 2000
};


const W: usize = 9;
const H: usize = 7;

const POLICY_SIZE: usize = W;
const INPUT_SIZE: usize = H * W * 2;

const cpuct: f64 = 4.0;


fn main() {
    let mut a = [1.; POLICY_SIZE];
    dirichlet_noise(&mut a);
    let mut mcts = MCTS::new();

    let (a, b) = mcts.GetAction(Instant::now() + Duration::from_millis(10000));
    println!("Hello, world!");
}
