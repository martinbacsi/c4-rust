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

    fn update_with_action(&mut self, action: u8) {
        let mut new_root = Option::None;
        self.root.children.drain(..).for_each(|c: Box<Node>| {
            if c.as_ref().game.last_move == action {
                new_root = Some(c);
            } else {
                self.pool.push(c);
            }
        });
        mem::swap(new_root.as_mut().unwrap(), &mut self.root);
        self.pool.push(new_root.unwrap());
    }

    fn get_move_probs_selfplay(&mut self) -> (u8, [f64; POLICY_SIZE]) {
        for i in 0..conf.iters {
            self.root.playout(&mut self.nn, &mut self.pool);
        }
        let mut p = self.root.prob_vector();
        dirichlet_noise(&mut p);

        let mut best = 0.0;
        let mut a = u8::MAX;
        self.root.children.iter().for_each(|c| {
            let d = p[c.game.last_move as usize] * rand_float();
            if d > best {
                best = d;
                a = c.game.last_move;
            } 
        });
        (a, p)
    }

    fn get_move_probs_play(&mut self,  endt: Instant) -> (u8, [f64; POLICY_SIZE]) {
        while Instant::now() < endt {
            self.root.playout(&mut self.nn, &mut self.pool);
        }
        let p = self.root.prob_vector();
        let mut a = self.root.children[0].game.last_move;
        self.root.children.iter().for_each(|c|{
            if p[c.game.last_move as usize] > p[self.root.children[a as usize].game.last_move as usize] {
                a = c.game.last_move;
            }
        });
        (a, p)
    }

    pub fn self_play(&mut self) {
        while self.root.game.outcome == Outcome::None {
            let (a, p) = self.get_move_probs_selfplay();
            self.update_with_action(a);
        }
    }

    pub fn play_against(&mut self) {
        while self.root.game.outcome == Outcome::None {
            let (a, _) = self.get_move_probs_play( Instant::now() + Duration::from_millis(50));
            self.update_with_action(a);

            self.root.game.print();
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).expect("read error");
            let n = buffer.trim().parse::<u8>().unwrap();

            self.update_with_action(n);
        }
        self.root.game.print();
    }
}
