use crate::connect4::HEIGHT;
use crate::Connect4;
use crate::NNManager;
use crate::Outcome;
use crate::Pool;
use crate::CONF;
use crate::POLICY_SIZE;

#[derive(Clone)]
pub struct Node {
    pub terminal: bool,
    pub visits: i32,
    pub value: i8,
    pub q: f64,
    p: f32,
    pub children: Vec<Box<Node>>,
    pub game: Connect4,
    expanded: bool,
    live_child: u16,
}

impl Node {
    pub fn new() -> Self {
        Node {
            terminal: false,
            visits: 0,
            value: -1,
            q: 0.,
            p: 0.,
            children: Vec::new(),
            game: Connect4::new(),
            expanded: false,
            live_child: 0,
        }
    }

    pub fn reinit(&mut self) {
        self.terminal = false;
        self.visits = 0;
        self.value = -1;
        self.q = 0.;
        self.p = 0.;
        self.children.clear();
        self.game = Connect4::new();
        self.expanded = false;
        self.live_child = 0;
    }

    fn ucb(&self, mult: f64) -> f64 {
        ((self.p as f64) * mult + self.q) / ((1 + self.visits) as f64)
    }

    pub fn select(&mut self, nn: &mut NNManager, pool: &mut Pool) -> usize {
        if self.children.is_empty() {
            let nnval = nn.get(&self.game);
            (0..POLICY_SIZE).for_each(|a| {
                if self.game.height[a] < HEIGHT as u8 {
                    self.live_child |= 1 << a;
                    let mut n = pool.pop();
                    n.p = nnval.p[a];
                    n.game = self.game;
                    n.game.step(a as u8);
                    self.children.push(n);
                };
            });
        }

        let mult = CONF.cpuct * (self.visits as f64).sqrt();
        let mut best_val = f64::NEG_INFINITY;
        let mut best = 0;
        for cid in 0..self.children.len() {
            let c = &self.children[cid];
            let val = c.ucb(mult);
            if val > best_val {
                best_val = val;
                best = cid;
            }
        }
        best
    }

    fn expand(&mut self, nn: &mut NNManager) -> f32 {
        let nnval = nn.get(&self.game);
        self.expanded = true;
        nnval.v
    }

    fn update(&mut self, value: f32) {
        self.visits += 1;
        self.q += value as f64;
    }

    pub fn prob_vector(&self) -> [f32; POLICY_SIZE] {
        let mut probs: [f32; POLICY_SIZE] = [0.0; POLICY_SIZE];
        let mut sum = 0f32;
        if self.terminal {
            for c in self.children.iter() {
                if c.terminal && c.value == -self.value {
                    probs[c.game.last_move as usize] = 1.0;
                    sum += 1.0;
                }
            }
        } else {
            for c in self.children.iter() {
                probs[c.game.last_move as usize] = c.visits as f32;
                sum += probs[c.game.last_move as usize];
            }
        }
        probs.iter_mut().for_each(|p| *p /= sum);
        probs
    }

    pub fn playout(&mut self, nn: &mut NNManager, pool: &mut Pool) -> f32 {
        if self.game.outcome != Outcome::None {
            self.value = if self.game.outcome == Outcome::Win {
                1
            } else {
                0
            };
            self.terminal = true;
        }
        let val;

        if !self.expanded {
            val = self.expand(nn);
        } else {
            let cid = self.select(nn, pool);
            if self.terminal {
                val = self.value as f32;
            } else {
                let c = &mut self.children[cid];
                val = -c.playout(nn, pool);
                if c.terminal {
                    self.value = self.value.max(c.value);
                    if c.value == 1 {
                        self.live_child = 0;
                    } else {
                        self.live_child ^= 1 << c.game.last_move;
                    }
                    if self.live_child == 0 {
                        self.terminal = true;
                        self.value = -self.value;
                    }
                }
            }
        }
        self.update(val);
        val
    }
}
