
use std::cmp::max;
use std::mem;
use std::{collections::HashMap};
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
    pub children: Vec<Box<Node>>,
    game: Connect4
}

impl Node {
    fn new()-> Self  {
        Node {
            terminal: false,
            visits: 0,
            value: -1.,
            Q: 0.,
            P: 0.,
            children: Vec::new(),
            game: Connect4::new()
        }
    }

    fn reinit(&mut self){
        *self = Node::new();
    }

    fn UCB(&self, mult: f64) -> f64 {
        ((self.P as f64) * mult + self.Q) / ((1 + self.visits) as f64)
    }

    fn Select(&mut self) -> usize {
        if self.terminal {
            0 as usize
        } else {
            let mult = cpuct * (self.visits as f64).sqrt();
            let mut best_val = f64::NEG_INFINITY;
            let mut best = 0;
            self.terminal = true;
            for cid in 0..self.children.len() {
                let c = &self.children[cid];
                if c.terminal {
                    let v = -c.value;
                    if v > self.value {
                        self.value = v;
                    }
                } else {
                    self.terminal = false;
                    let val = c.UCB(mult);
                    if val > best_val {
                        best_val = val; 
                        best = cid;
                    }
                }
            }
            best
        } 
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

   fn Update(&mut self, value: f32) {
        self.visits += 1;
        self.Q += value as f64;
   }

   fn PlayOut(&mut self, NN: &mut NNManager, pool: &mut Pool) -> f32 {
        if self.game.outcome != Outcome::None {
            self.value =  if self.game.outcome == Outcome::Win {1.}  else {0.};
            self.terminal = true;
        }
        let mut val = f32::NAN;

        if self.children.is_empty() {
            val = self.Expand(NN, pool);
        } else {
            let cid = self.Select();
            if self.terminal {
                val = self.value;
            } else {
                val = -self.children[cid].PlayOut(NN, pool);
            } 
        }
        self.Update(val);
        val
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
        self.nodes.pop().unwrap()
    }

    fn push(&mut self, mut ptr: Box<Node>) {
        ptr.children.drain(..).map(|n|self.push(n));
        ptr.reinit();
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

    fn UpdateWithAction(&mut self, action: u8) {
        let mut newRoot = Option::None;
        self.root.children.drain(..).map(| c|{
            if c.game.lastMove == action {
                newRoot = Some(c);
            } else {
                self.pool.push(c);
            }
        });
        mem::swap(newRoot.as_mut().unwrap(), &mut self.root);
        self.pool.push(newRoot.unwrap());
    }
}

fn main() {
    let mcts = MCTS::new();
    println!("Hello, world!");
}
