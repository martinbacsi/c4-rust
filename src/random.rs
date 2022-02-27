use std::ops::{Mul, Add};

use crate::POLICY_SIZE;

pub struct rand {
    x: usize,
    y: usize,
    z: usize
}

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

impl rand {
    pub fn xorshf96(&mut self) -> usize {
        self.x ^=  &self.x << 16;
        self.x ^= self.x >> 5;
        self.x ^= self.x << 1;
        let mut t= self.x;
        self.x = self.y;
        self.y = self.z;
        self.z = t ^ self.x ^ self.y;
        self.z
    }

    pub fn rand_gamma(&mut self, x: f64, a: f64, b: f64) -> f64 {
        gammaPdf(x * self.xorshf96() as f64 / usize::MAX as f64, a, b)
    }

    pub fn new() -> rand {
        rand{
            x:123456789, 
            y:362436069, 
            z:521288629
        }
    }  

    pub fn dirichlet_noise(&mut self, v: &mut [f64; POLICY_SIZE]) {
        //TODO PARAM, CHECK
        let dir: [f64; POLICY_SIZE] = [(); POLICY_SIZE].map(|_| self.rand_gamma(1.0, 0.5, 0.5) );
        let sum: f64 = dir.iter().sum();

        for (i,e) in v.iter_mut().enumerate() {
            e.mul(1. - DIRICHLET_EPS);
            e.add(DIRICHLET_EPS * dir[i] / sum);
        }
    }
}
