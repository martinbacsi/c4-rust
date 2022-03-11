use std::arch::x86_64::_rdrand64_step;

use crate::POLICY_SIZE;

const PI: f32 = 3.14159265358979323846264338327950288f32;
//http://www.rskey.org/gamma.htm
fn gamm(x: f32) -> f32 {
    let ret = 1.000000000190015
        + 76.18009172947146 / (x + 1.)
        + -86.50532032941677 / (x + 2.)
        + 24.01409824083091 / (x + 3.)
        + -1.231739572450155 / (x + 4.)
        + 1.208650973866179e-3 / (x + 5.)
        + -5.395239384953e-6 / (x + 6.);

    ret * f32::sqrt(2. * PI) / x * f32::powf(x + 5.5, x + 0.5) * f32::exp(-x - 5.5)
}

fn gamma_pdf(x: f32, a: f32, b: f32) -> f32 {
    assert!(x > 0. && a > 0. && b > 0.);
    f32::exp(-x * b) * f32::powf(x, a - 1.0) * f32::powf(b, a) / gamm(a)
}

pub fn rand() -> u64 {
    let mut r: u64 = 0;
    unsafe {
        assert!(_rdrand64_step(&mut r) == 1);
    }
    r
}

pub fn rand_float() -> f32 {
    rand() as f32 / u64::MAX as f32
}

pub fn rand_gamma(x: f32, a: f32, b: f32) -> f32 {
    gamma_pdf(x * rand_float(), a, b)
}

pub fn dirichlet_noise() -> [f32; POLICY_SIZE] {
    //TODO PARAM, CHECK
    let mut dir: [f32; POLICY_SIZE] = [0.0; POLICY_SIZE];
    dir.iter_mut().for_each(|a| *a = rand_gamma(1.0, 0.5, 1.0));
    dir
}
