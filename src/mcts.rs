use crate::game::Action;
use crate::game::Game;
use crate::game::*;
use crate::nn::NN;
use crate::node;
#[cfg(target_os = "windows")]
use crate::random::dirichlet_noise;
use crate::sample::Sample;
use crate::Node;
use crate::CONF;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::mem::swap;
use std::time::Duration;
use std::{collections::HashMap, mem, time::Instant};
const DIRICHLET_EPS: f32 = 0.3;

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}
pub struct MCTS {
    roots: [Node; 2],
    nn: NN,
    r: ThreadRng,
}

impl MCTS {
    pub fn new() -> MCTS {
        let mut mcts = MCTS {
            roots: [Node::new(), Node::new()],
            nn: NN::new(),
            r: rand::thread_rng(),
        };
        mcts
    }
    fn update_with_action(&mut self, player: usize, action: usize) {
        let root = &mut self.roots[player];
        assert!(!root.children.is_empty());
        let mut new_root = Node::new();
        while root.children.len() > 0 {
            let r = root.children.pop().unwrap();
            if r.action == action {
                new_root = r;
            }
        }
        *root = new_root;
    }

    fn playout(&mut self, _game: &Game) {
        let mut game = _game.clone();
        let mut leaf = false;
        let mut val = -69.0;

        let mut stacks = [Vec::new(), Vec::new()];
        unsafe {
            for i in 0..2 {
                stacks[i].push(&mut self.roots[i] as *mut Node);
            }

            while !leaf && !game.is_game_over() {
                let mut a = [69; 69];
                for i in 0..2 {
                    let st = *stacks[i].last().unwrap();
                    if !(*st).expanded {
                        val = (*st).expand(&mut self.nn, &game, i);
                        if i == 1 {
                            val = -val;
                        }
                        //eprintln!("{}", val);
                        leaf = true;
                    } else {
                        a[i] = (*st).select();
                        stacks[i].push(&mut (*st).children[a[i]]);
                    }
                }

                if !leaf {
                    game.step([Action::new(a[0], false), Action::new(a[1], true)]);
                }

                //eprintln!("{}", game.game_turn);
            }
            if game.is_game_over() {
                val = game.score(0);
            }

            for i in 0..stacks[0].len().min(stacks[1].len()) {
                (**stacks[0].get(i).unwrap()).update(val);
                (**stacks[1].get(i).unwrap()).update(-val);
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn get_move_probs_selfplay(&mut self, player: usize) -> (usize, [f32; ACTION_SIZE]) {
        let mut p = self.roots[player].prob_vector();

        let dir = dirichlet_noise(&mut self.r);
        let mut sum = 0f32;
        for c in self.roots[player].children.iter() {
            let i = c.action as usize;
            sum += dir[i];
        }
        for c in self.roots[player].children.iter() {
            let i = c.action as usize;
            p[i] = p[i] * (1. - DIRICHLET_EPS) + DIRICHLET_EPS * dir[i] / sum;
        }

        let mut best = 0.0;
        let mut a = usize::MAX;
        self.roots[player].children.iter().for_each(|c| {
            let mut d = p[c.action as usize] * self.r.gen_range(0.0..1.0);

            if d > best {
                best = d;
                a = c.action;
            }
        });
        (a, p)
    }

    fn get_move_probs_play(&mut self, endt: Instant) -> u8 {
        while Instant::now() < endt {
            self.root.playout(&mut self.nn, &mut self.pool);
        }
        let a = self.root.children.iter().max_by_key(|b| {
            if self.root.terminal {
                b.value as i32
            } else {
                b.visits
            }
        });
        eprintln!("root visits: {}", self.root.visits);
        a.unwrap().game.last_move
    }

    #[cfg(target_os = "windows")]
    pub fn self_play(&mut self, ss: &mut Vec<Sample>) {
        use rand::RngCore;
        let mut game = Game::new(self.r.next_u64() as i64);
        let mut new_samples = Vec::new();
        while !game.is_game_over() {
            for _ in 0..CONF.iters {
                self.playout(&game);
            }
            let mut actions = [69, 69];
            //eprintln!(".................{}", game.game_turn);
            for i in 0..2 {
                let (a, p) = self.get_move_probs_selfplay(i);
                //for pp in p.iter() {
                //    eprint!("{} ", pp );
                //}
                //eprintln!();
                actions[i] = a;
                let mut sample = Sample::new(&self.roots[i], &game, i);
                sample.p.clone_from(&p);
                new_samples.push(sample);
                self.update_with_action(i, a);
            }
            game.step([
                Action::new(actions[0], false),
                Action::new(actions[1], true),
            ]);
        }
        for (i, s) in new_samples.iter_mut().enumerate() {
            s.v = game.score(i % 2);
            ss.push(*s);
        }

        eprintln!(
            "truns: {}, scores: {} vs {}. val:{}, ku_val: {} vs {}",
            game.game_turn,
            game.compute_player_score(0),
            game.compute_player_score(1),
            self.nn.run_game(&game, 0).v,
            self.roots[0].q,
            self.roots[1].q
        );
    }
}
