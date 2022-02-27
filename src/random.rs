use std::{ops::{Mul, Add}, arch::x86_64::_rdrand64_step};

use crate::POLICY_SIZE;


const DIRICHLET_EPS:f64 = 0.3;
const PI: f64 = 3.14159265358979323846264338327950288f64;
//http://www.rskey.org/gamma.htm   
fn gamm(x: f64) -> f64 {
    let ret = (1.000000000190015 + 
                 76.18009172947146 / (x + 1.) +  
                 -86.50532032941677 / (x + 2.) + 
                 24.01409824083091 / (x + 3.) +  
                 -1.231739572450155 / (x + 4.) + 
                 1.208650973866179e-3 / (x + 5.) + 
                 -5.395239384953e-6 / (x + 6.));
    
    ret * f64::sqrt(2. * PI)/x * f64::powf(x + 5.5, x+0.5) * f64::exp(-x-5.5)
}

fn gammaPdf(x: f64, a: f64, b: f64) -> f64 {
    assert!(x > 0. && a > 0. && b > 0.);
    f64::exp(-x * b) * f64::powf(x, a - 1.0) * f64::powf(b, a) / gamm(a)
}


pub fn rand_gamma(x: f64, a: f64, b: f64) -> f64 {
    let mut r:u64 = 0;
    unsafe {
        assert!(_rdrand64_step(&mut r) == 1);
    }
    gammaPdf(x * r as f64 / u64::MAX as f64, a, b)
}

pub fn dirichlet_noise(v: &mut [f64; POLICY_SIZE]) {
    //TODO PARAM, CHECK
    let dir: [f64; POLICY_SIZE] = [(); POLICY_SIZE].map(|_| rand_gamma(1.0, 0.5, 1.0) );
    let sum: f64 = dir.iter().sum();

    for i in 0..POLICY_SIZE {
        v[i] = v[i] * (1. - DIRICHLET_EPS) + DIRICHLET_EPS * dir[i] / sum;
    }
}

