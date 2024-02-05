use rustfmt::config;
use tch::Device;
use tch::{nn,Tensor, nn::OptimizerConfig, nn::VarStore,};
use crate::{game::*, CONF};
use crate::sample::Sample;

use rand::prelude::*;
use crate::xorshift::*;

const GAMMA: f64 = 0.99;
const LAMBDA: f64 = 0.95;
const EPS_CLIP: f32 = 0.1;
const FISH_VECTOR_LENGTH: i64 = 6;
const BATCH_SIZE: i64 = 32;
use crate::game::*;
pub struct NnOutput {
    pub p: [f32; ACTION_SIZE],
    pub v: f32,
}
trait SwishExt {
    fn swish(&self) -> Tensor;
}

// Implement the Swish trait for Tensor.
impl SwishExt for Tensor {
    fn swish(&self) -> Tensor {
        self * self.sigmoid()
    }
}

pub struct NN {
    vs: VarStore,
    optimizer: nn::Optimizer<nn::Adam>,
    fc0: nn::Linear,
    fc1: nn::Linear,
    fc2: nn::Linear,
    fc3: nn::Linear,
    fc4: nn::Linear,
    fc5: nn::Linear,

    fc_pi: nn::Linear,
    fc_v: nn::Linear,
}

fn cross_entropy_loss(predictions: &Tensor, targets: &Tensor) -> Tensor {
    let elementwise_loss = -targets * predictions.log();
    let loss = elementwise_loss.sum1(&[1], false, predictions.kind());
    loss.mean(tch::Kind::Float)
}

impl NN {
    pub fn new() -> NN {
        let vs = VarStore::new(tch::Device::Cpu);
        let root = &vs.root();
        let optimizer = nn::Adam::default().build(&vs, CONF.learning_rate).unwrap();
        let fc0 = nn::linear(root / "c_fc0", INPUT_PER_FISH as i64, 32, Default::default());
        let fc1 = nn::linear(root / "c_fc1", 32, 32, Default::default());
        let fc2 = nn::linear(root / "c_fc2", 32, FISH_VECTOR_LENGTH, Default::default());

        let fc3 = nn::linear(root / "c_fc3", 2 * GLOBAL_INPUTS as i64 + FISH_VECTOR_LENGTH + 2 * INPUT_PER_DRONE as i64 * Game::DRONES_PER_PLAYER as i64, 128, Default::default());
        let fc4 = nn::linear(root / "c_fc4", 128, 128, Default::default());
        let fc5 = nn::linear(root / "c_fc5", 128, 128, Default::default());
        let fc_pi = nn::linear(root / "c_fc_pi", 128, ACTION_SIZE as i64, Default::default());
        let fc_v= nn::linear(root / "c_fc_v", 128, 1, Default::default());

        let mut ppo = NN {
            vs,
            optimizer,
            fc0,
            fc1,
            fc2,
            fc3,
            fc4,
            fc5,
            fc_pi,
            fc_v,
        };
        ppo.optimizer.set_weight_decay(1e-5);
        if CONF.load_file {
            ppo.vs.load("model_fish.pt").expect("failed to load");   
        }
        ppo
    }

    pub fn base_net(&self, x: &Tensor) -> Tensor {
        let splt = x.split(INPUT_PER_FISH as i64 * 12, -1);
        let fishes = splt.get(0).unwrap();
        let drone = splt.get(1).unwrap();
        let fishes_batch = &fishes.reshape(&[-1, INPUT_PER_FISH as i64]);
        let fishes_processed = fishes_batch.apply(&self.fc0)
        .swish()
        .apply(&self.fc1)
        .swish()
        .apply(&self.fc2)
        .swish();
        //eprintln!("kamaty");

       
        //s
    

        let fishes_backshaped = fishes_processed.view([-1,  12, FISH_VECTOR_LENGTH as i64 ]).permute(&[0, 2, 1]);

        //let fishes_backshaped = fishes_processed.view([-1,  12 * FISH_VECTOR_LENGTH as i64 ]);


      
        let fishes_pooled = fishes_backshaped.max_pool1d(&[12], &[12], &[0], &[1], false).view([-1, FISH_VECTOR_LENGTH]);

        let fishesprocess_and_drones =  Tensor::cat(&[&fishes_pooled, &drone], 1);
        

        fishesprocess_and_drones
        
        .apply(&self.fc3)
        .swish()
        .apply(&self.fc4)
        .swish()
        .apply(&self.fc5)
        .swish()
    }

    pub fn run(&self, x: &Tensor) -> (Tensor, Tensor) {   
        let a = &self.base_net(x);
        let pi = a.apply(&self.fc_pi).softmax(1, tch::Kind::Float);
        let v = 
        a.apply(&self.fc_v)
        .tanh();
        (pi, v)
    }

    pub fn run_game(&self, game: &Game, player: usize) -> NnOutput {
        let state = game.encode(player);
        let (pi_t, v_t) =  self.run(&Tensor::of_slice(&state).view([1, STATE_SIZE as i64]));
        let mut pi = [0.0; ACTION_SIZE];
        unsafe {
            let pi_ptr = pi_t.data_ptr() as *const f32;
            for i in 0..ACTION_SIZE {
                pi[i] = *pi_ptr.offset(i as isize);
            }
        }
        NnOutput {
            p: pi,
            v: v_t.double_value(&[0, 0]) as f32
        }
    }
   

    
    pub fn train(&mut self, samples: &Vec<Sample>) {
        let i = Tensor::zeros(&[BATCH_SIZE, STATE_SIZE as i64], (tch::Kind::Float, tch::Device::Cpu));      
        let pi = Tensor::zeros(&[BATCH_SIZE, ACTION_SIZE as i64], (tch::Kind::Float, tch::Device::Cpu));     
        let v = Tensor::zeros(&[BATCH_SIZE, 1], (tch::Kind::Float, tch::Device::Cpu));     

        for _ in 0..10 {
            unsafe {
                let i_data =  i.data_ptr() as *mut f32;
                let pi_data =  pi.data_ptr() as *mut f32;
                let v_data =  v.data_ptr() as *mut f32;
                for i in 0..BATCH_SIZE as isize {
                    let sample = samples.choose(&mut rand::thread_rng()).unwrap();
                    for j in 0..STATE_SIZE {
                        *i_data.offset(i * STATE_SIZE as isize + j as isize) = sample.input[j];
                    }
                    for j in 0..ACTION_SIZE {
                        *pi_data.offset(i * ACTION_SIZE as isize + j as isize) = sample.p[j];
                    }
                    *v_data.offset(i as isize) = sample.v;
                }
            }
            let (pi_val, v_val) = self.run(&i);
            //eprintln!("........................");
            //v.print();
            //v_val.print();
            let l2 =  (&v - &v_val).pow(2.0).mean(tch::Kind::Float);
            let ent = (1.0 / ACTION_SIZE as f32) * &cross_entropy_loss(&pi.softmax(1, tch::Kind::Float), &pi_val);
            let loss = l2 + ent;
            self.optimizer.zero_grad();
            loss.print();
            loss.backward();
            self.optimizer.step();
        }
        //eprintln!("trained");
        self.vs.save("model_fish.pt").expect("failed to load");
    }

}