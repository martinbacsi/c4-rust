#[cfg(target_os = "windows")]
use rand_distr::Gamma;
#[cfg(target_os = "windows")]
use rand::{distributions::Distribution, Rng};
#[cfg(target_os = "windows")]
use crate::game::ACTION_SIZE;
#[cfg(target_os = "windows")]
pub fn dirichlet_noise<R: Rng>(rng: &mut R) -> [f32; ACTION_SIZE] {
    let gamma = Gamma::new(0.5, 1.0).unwrap();
    let mut dir: [f32; ACTION_SIZE] = [0.0; ACTION_SIZE];
    dir.iter_mut().for_each(|a| *a = gamma.sample(rng));
    dir
}
