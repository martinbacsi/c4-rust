use crate::Connect4;
use crate::cpuct;
use crate::NNManager;
use crate::Pool;
use crate::Outcome;

#[derive(Clone)]
pub struct Node {
    terminal: bool,
    pub visits: i32,
    value: f32,
    Q: f64,
    P: f32,
    pub children: Vec<Box<Node>>,
    pub game: Connect4
}

impl Node {
    pub fn new()-> Self  {
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

    pub fn reinit(&mut self){
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

   pub fn PlayOut(&mut self, NN: &mut NNManager, pool: &mut Pool) -> f32 {
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
