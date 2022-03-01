use crate::cpuct;
use crate::Connect4;
use crate::NNManager;
use crate::Outcome;
use crate::Pool;
use crate::POLICY_SIZE;

#[derive(Clone)]
pub struct Node {
    terminal: bool,
    pub visits: i32,
    value: f32,
    Q: f64,
    P: f32,
    pub children: Vec<Box<Node>>,
    pub game: Connect4,
}

impl Node {
    pub fn new() -> Self {
        Node {
            terminal: false,
            visits: 0,
            value: -1.,
            Q: 0.,
            P: 0.,
            children: Vec::new(),
            game: Connect4::new(),
        }
    }

    pub fn reinit(&mut self) {
        *self = Node::new();
    }

    fn ucb(&self, mult: f64) -> f64 {
        ((self.P as f64) * mult + self.Q) / ((1 + self.visits) as f64)
    }

    fn select(&mut self) -> usize {
        if self.terminal {
            0 as usize
        } else {
            let mult = (cpuct * self.visits as f64).sqrt();
            let mut best_val = f64::NEG_INFINITY;
            let mut best = 0;
            self.terminal = true;
            self.value = -1.;
            for cid in 0..self.children.len() {
                let c = &self.children[cid];
                self.value = f32::max(self.value, c.value);
                if c.terminal && c.value == 1.0 {
                    self.terminal = true;
                    break;
                } else {
                    self.terminal = false;
                }
                let val = c.ucb(mult);
                if val > best_val {
                    best_val = val;
                    best = cid;
                }
            }
            self.value = -self.value;
            best
        }
    }

    fn expand(&mut self, NN: &mut NNManager, pool: &mut Pool) -> f32 {
        let nnval = NN.get(&self.game);
        self.game.on_valid_action(&mut |a| {
            let mut n = pool.pop();
            n.P = nnval.p[a as usize];
            n.game = self.game;
            n.game.step(a);
            self.children.push(n);
        });
        nnval.v
    }

    fn update(&mut self, value: f32) {
        self.visits += 1;
        self.Q += value as f64;
    }

    pub fn prob_vector(&self) -> [f64; POLICY_SIZE] {
        let mut probs: [f64; POLICY_SIZE] = [0.0; POLICY_SIZE];
        if self.terminal {
            let max = self
                .children
                .iter()
                .fold(0.0, |max, c| if c.value > max { c.value } else { max });
            let mut sum = 0.0;
            for c in self.children.iter() {
                if c.value == max {
                    sum += 1.0;
                    probs[c.game.last_move as usize] = 1.0;
                }
            }
            probs.iter_mut().for_each(|p| *p /= sum);
        } else {
            let all_visits = (&self.children)
                .iter()
                .fold(0, |all_visits, x| all_visits + x.visits);
            (&self.children).into_iter().for_each(|n| {
                probs[n.game.last_move as usize] = n.visits as f64 / all_visits as f64
            });
        }

        probs
    }

    pub fn playout(&mut self, NN: &mut NNManager, pool: &mut Pool) -> f32 {
        if self.game.outcome != Outcome::None {
            self.value = if self.game.outcome == Outcome::Win {
                1.
            } else {
                0.
            };
            self.terminal = true;
        }
        let val;

        if self.children.is_empty() {
            val = self.expand(NN, pool);
        } else {
            let cid = self.select();
            if self.terminal {
                val = self.value;
            } else {
                val = -self.children[cid].playout(NN, pool);
            }
        }
        self.update(val);
        val
    }
}
