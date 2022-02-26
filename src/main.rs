
use std::{collections::HashMap, hash::Hash, ops::RangeBounds, sync::Arc};
use connect4::Connect4;
use connect4::Outcome;

mod connect4;

const cpuct: f64 = 4.0;

const W: usize = 9;
const H: usize = 7;

const POLICY_SIZE: usize = W;
const INPUT_SIZE: usize = H * W * 2;

struct NnOutput {
    p: [f32; POLICY_SIZE],
    v: f32
}
struct NN {

}

struct NNManager {
    cache: HashMap<usize, NnOutput>
}

impl NNManager {
    fn Get(&mut self, game: &Connect4) -> &NnOutput {
        let hash = game.hash();
        if !self.cache.contains_key(&hash) {
            self.cache.insert(hash,NnOutput{p: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0., 0.0, 0.0], v: 0.0});
        }
        &self.cache[&hash]
    }
}

#[derive(Clone)]
pub struct Node {
    terminal: bool,
    visits: i32,
    value: f32,
    Q: f64,
    P: f32,
    children: Vec<Box<Node>>,
    game: Connect4
}

impl Node {
    fn new()-> Self  {
        Node {
            terminal: false,
            visits: 0,
            value: 0.,
            Q: 0.,
            P: 0.,
            children: Vec::new(),
            game: Connect4::new()
        }
    }

    fn UCB(&self, mult: f64) -> f64 {
        ((self.P as f64) * mult + self.Q) / ((1 + self.visits) as f64)
    }

    fn Select(&self) -> &Box<Node> {
        assert!(!self.terminal);
        let mult = cpuct * (self.visits as f64).sqrt();
        let mut best_val = f64::NEG_INFINITY;
        let mut best = &self.children[0];

        for c in self.children.iter() {
            let val = c.UCB(mult);
            if val > best_val {
                best_val = val;
                //TODO itt felülírja vagy mi a szösz
                best = c;
            }
        }
        best
   }

   fn Expand(&mut self, NN: &mut NNManager, pool: &mut Pool) -> f32 {
        let nnval = NN.Get(&self.game);
        self.game.OnValidActions(&mut |a| {
            let mut n = pool.pop();
            n.P = nnval.p[a as usize];
            n.game = self.game;
            n.game.step(a);
            self.children.push(n);
        });
        nnval.v
   }

   fn PlayOut(&mut self, NN: &mut NNManager, pool: &mut Pool) -> f32 {
        if self.game.outcome != Outcome::None {
            self.value =  if self.game.outcome == Outcome::Win {1.}  else {0.};
            self.terminal = true;
        }

        if self.terminal {

        }


        0.0

   }
}

pub struct Pool {
    nodes: Vec<Box<Node>>,
}

impl Pool {
    fn new(cap: usize) -> Pool{
        let mut p = Pool{
            nodes: Vec::new()
        };
        p.nodes.resize(cap, Box::new(Node::new()));
        p
    }

    fn pop(&mut self) -> Box<Node> {
        let mut ret = self.nodes.pop().unwrap();
        (*ret) = Node::new();
        ret
    }

    fn push(mut self, ptr: Box<Node>) {
        self.nodes.push(ptr);
    }
}

struct MCTS {
    pool: Pool,
    root: Box<Node>
}

impl MCTS {
    fn new() -> MCTS {
        let mcts = MCTS {
            pool: Pool::new(1000000),
            root: Box::new(Node::new())
        };
        mcts
    }
}

fn main() {
    let mcts = MCTS::new();
    println!("Hello, world!");
}
