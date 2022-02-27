
mod connect4;
mod nn;
mod node;
mod mcts;
mod pool;
mod random;
use std::mem;

use std::{collections::HashMap};
use nn::NNManager;
use node::Node;
use connect4::Connect4;
use connect4::Outcome;
use std::time::{Instant};
use mcts::MCTS;
use pool::Pool;
use random::rand;


const W: usize = 9;
const H: usize = 7;

const POLICY_SIZE: usize = W;
const INPUT_SIZE: usize = H * W * 2;

const cpuct: f64 = 4.0;


fn main() {
    let mcts = MCTS::new();
    println!("Hello, world!");
}
