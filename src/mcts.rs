use crate::NNManager;
use crate::Pool;
use crate::Node;
use crate::POLICY_SIZE;
use crate::conf;
use crate::connect4::Outcome;
use crate::nn::NN;
use crate::random::dirichlet_noise;
use crate::random::rand_float;
use std::time::Duration;
use std::{collections::HashMap, mem, time::Instant};
use std::io;
pub struct MCTS {
    pool: Pool,
    root: Box<Node>,
    nn: NNManager
}

impl MCTS {
    pub fn new() -> MCTS {
        let mut mcts = MCTS {
            pool: Pool::new(1000000),
            root: Box::new(Node::new()), 
            nn: NNManager{
                cache: HashMap::new(),
                nn: NN::new()
            }
        };
        mcts.nn.nn.read_weights("best.w32");
        mcts
    }

    fn UpdateWithAction(&mut self, action: u8) {
        let mut newRoot = Option::None;
        while self.root.children.len() > 0 {
            if self.root.children.last().unwrap().game.lastMove == action {
                newRoot = Some(self.root.children.pop().unwrap());   
            } else {
                self.pool.push(self.root.children.pop().unwrap());
            }
        }
        mem::swap(newRoot.as_mut().unwrap(), &mut self.root);
        self.pool.push(newRoot.unwrap());
    }

    fn get_move_probs_selfplay(&mut self) -> (u8, [f64; POLICY_SIZE]) {
        for i in 0..conf.iters {
            self.root.PlayOut(&mut self.nn, &mut self.pool);
        }
        let mut p = self.root.prob_vector();
        dirichlet_noise(&mut p);

        let mut best = 0.0;
        let mut a = u8::MAX;
        self.root.children.iter().for_each(|c| {
            let d = p[c.game.lastMove as usize] * rand_float();
            if d > best {
                best = d;
                a = c.game.lastMove;
            } 
        });
        (a, p)
    }

    fn get_move_probs_play(&mut self,  endt: Instant) -> (u8, [f64; POLICY_SIZE]) {
        while Instant::now() < endt {
            self.root.PlayOut(&mut self.nn, &mut self.pool);
        }
        let p = self.root.prob_vector();
        let mut a = self.root.children[0].game.lastMove;
        self.root.children.iter().for_each(|c|{
            if p[c.game.lastMove as usize] > p[self.root.children[a as usize].game.lastMove as usize] {
                a = c.game.lastMove;
            }
        });
        (a, p)
    }

    pub fn self_play(&mut self) {
        while self.root.game.outcome == Outcome::None {
            let (a, p) = self.get_move_probs_selfplay();
            self.UpdateWithAction(a);
        }
    }

    pub fn play_against(&mut self) {
        while self.root.game.outcome == Outcome::None {
            let (a, p) = self.get_move_probs_play( Instant::now() + Duration::from_millis(50));
            self.UpdateWithAction(a);

            self.root.game.print();
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer);
            let n = buffer.trim().parse::<u8>().unwrap();

            self.UpdateWithAction(n);
        }
        self.root.game.print();
    }
}
