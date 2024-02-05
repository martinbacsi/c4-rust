use crate::game::*;
use crate::nn::NN;
use crate::CONF;

#[derive(Clone)]
pub struct Node {
    pub visits: i32,
    pub q: f64,
    p: f32,
    pub children: Vec<Node>,
    pub action: usize,
    pub expanded: bool,
    pi: [f32; ACTION_SIZE]
}

impl Node {
    pub fn new() -> Self {
        Node {
            visits: 0,
            q: 0.,
            p: 0.,
            children: Vec::new(),
            action: 420,
            expanded: false,
            pi: [0.0; ACTION_SIZE]
        }
    }

    pub fn reinit(&mut self) {
        *self = Node::new();
    }

    fn ucb(&self, mult: f64) -> f64 {
        ((self.p as f64) * mult + self.q) / ((1 + self.visits) as f64)
    }

    pub fn select(&mut self) -> usize {
        if self.children.is_empty() {
            (0..ACTION_SIZE).for_each(|a| {
                let mut n = Node::new();
                n.p = self.pi[a];
                n.action = a;
                self.children.push(n);
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

    pub fn expand(&mut self, nn: &NN, game: &Game, player: usize) -> f32 {
        let nnval = nn.run_game(game, player);
        self.pi = nnval.p;
        self.expanded = true;
        nnval.v
    }

    pub fn update(&mut self, value: f32) {
        self.visits += 1;
        self.q += value as f64;
    }

    pub fn prob_vector(&self) -> [f32; ACTION_SIZE] {
        let mut probs: [f32; ACTION_SIZE] = [0.0; ACTION_SIZE];
        let mut sum = 0f32;
      
        for c in self.children.iter() {
            probs[c.action] = c.visits as f32;
            sum += probs[c.action as usize];
        }
        
        probs.iter_mut().for_each(|p| *p /= sum);
        probs
    }
}


