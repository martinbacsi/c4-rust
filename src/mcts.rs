use crate::NNManager;
use crate::Pool;
use crate::Node;
use crate::POLICY_SIZE;
use crate::conf;
use crate::random::dirichlet_noise;
use crate::random::rand_float;
use std::{collections::HashMap, mem, time::Instant};
pub struct MCTS {
    pool: Pool,
    root: Box<Node>,
    nn: NNManager
}

impl MCTS {
    pub fn new() -> MCTS {
        let mcts = MCTS {
            pool: Pool::new(1000000),
            root: Box::new(Node::new()), 
            nn: NNManager{
                cache: HashMap::new()
            }
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

    fn GetMoveProbs(&mut self, endt: Instant) -> [f64; POLICY_SIZE]{
        let mut probs: [f64; POLICY_SIZE]  = [0.; POLICY_SIZE];
        if conf.selfplay {
            let mut i = 0;
            while i < conf.iters && Instant::now() < endt {
                self.root.PlayOut(&mut self.nn, &mut self.pool);
                i += 1;
            }
        } 
        let all_visits = (&self.root.children).iter().fold(0, |all_visits, x| all_visits + x.visits);
        (&self.root.children).into_iter().for_each(|n| probs[n.game.lastMove as usize] = n.visits as f64 / all_visits as f64);
        probs
    }

    pub fn GetAction(&mut self, endt: Instant) -> (u8, [f64; POLICY_SIZE]) {
        let mut probs = self.GetMoveProbs(endt);
        let mut a = 0;
        if conf.selfplay {
            dirichlet_noise(&mut probs);
            let mut best = 0.;
            self.root.children.iter().for_each(|c|{
                let p = probs[c.game.lastMove as usize] * rand_float() ;
                if p > best {
                    best = p;
                    a = c.game.lastMove;
                } 
            });
        } else {
            let b: &Box<Node> = self.root.children.iter().fold( &self.root.children[0], |a, c|{
                if probs[c.game.lastMove as usize] > probs[a.game.lastMove as usize] {
                    c
                } else {
                    a
                }
            });
            a = b.game.lastMove;
        }
        (a, probs)
    }
}
