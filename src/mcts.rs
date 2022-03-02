use crate::conf;
use crate::connect4::Outcome;
use crate::nn::NN;
use crate::random::dirichlet_noise;
use crate::random::rand_float;
use crate::NNManager;
use crate::Node;
use crate::Pool;
use crate::POLICY_SIZE;
use std::fmt::Result;
use std::io;
use std::num::ParseIntError;
use std::time::Duration;
use std::{collections::HashMap, mem, time::Instant};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}
pub struct MCTS {
    pool: Pool,
    root: Box<Node>,
    nn: NNManager,
}

impl MCTS {
    pub fn new() -> MCTS {
        let mut mcts = MCTS {
            pool: Pool::new(1000000),
            root: Box::new(Node::new()),
            nn: NNManager {
                cache: HashMap::new(),
                nn: NN::new(),
            },
        };
        mcts.nn.nn.read_weights();
        for i in 0..2 {
            mcts.root.playout(&mut mcts.nn, &mut mcts.pool);
        }
        mcts
    }

    fn update_with_action(&mut self, action: u8) {
        let mut new_root = Option::None;
        while self.root.children.len() > 0 {
            if self.root.children.last().unwrap().game.last_move == action {
                new_root = Some(self.root.children.pop().unwrap());
            } else {
                self.pool.push(self.root.children.pop().unwrap());
            }
        }
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

    fn get_move_probs_play(&mut self, endt: Instant) -> (u8, [f64; POLICY_SIZE]) {
        while Instant::now() < endt {
            self.root.playout(&mut self.nn, &mut self.pool);
        }
        let p = self.root.prob_vector();
        let mut a = &self.root.children[0];
        self.root.children.iter().for_each(|c| {
            if p[c.game.last_move as usize] > p[a.game.last_move as usize] {
                a = c;
            }
        });
        eprintln!("root visits{}", self.root.visits);
        (a.game.last_move, p)
    }

    pub fn self_play(&mut self) {
        while self.root.game.outcome == Outcome::None {
            let (a, p) = self.get_move_probs_selfplay();
            self.update_with_action(a);
        }
    }

    pub fn play_against(&mut self) {
        for i in 0..64 {
            if i % 2 == 0 {
                self.root.game.print();
                loop {
                    let mut buffer = String::new();
                    std::io::stdin().read_line(&mut buffer).expect("read error");
                    let a_read = buffer.trim().parse::<u8>();
                    if a_read.is_ok() {
                        let a = a_read.unwrap();
                        if self
                            .root
                            .children
                            .iter()
                            .find(|c| c.game.last_move == a)
                            .is_some()
                        {
                            self.update_with_action(a);
                            break;
                        }
                    }
                    println!("hülye vagy");
                }
            } else {
                let (a, _) = self.get_move_probs_play(Instant::now() + Duration::from_millis(1000));
                self.update_with_action(a);
            }
            if self.root.game.outcome != Outcome::None {
                self.root.game.print();
                if i % 2 == 0 {
                    println!("nice!");
                } else {
                    println!("LOL XDDDD");
                }
                break;
            }
        }
    }

    pub fn cg(&mut self) {
        let mut endt;
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        for i in 0..64 {
            for _i in 0..10 + self.root.children.len() as usize {
                input_line.clear();
                io::stdin().read_line(&mut input_line).unwrap();
            }

            if i > 0 {
                endt = Instant::now() + Duration::from_millis(90);
            } else {
                endt = Instant::now() + Duration::from_millis(900);
            }
            let opp_previous_action = parse_input!(input_line, i32); // opponent's previous chosen column index (will be -1 for first player in the first turn)

            if opp_previous_action >= 0 {
                self.update_with_action(opp_previous_action as u8);
            }

            let (a, _p) = self.get_move_probs_play(endt);
            self.update_with_action(a);
            println!("{}", a);
        }
    }
}
