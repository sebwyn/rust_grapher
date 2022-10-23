use std::f32::consts::{PI, E};

use cgmath::num_traits::Pow;

pub struct Normal {
    pub deviation: f32,
    pub mean: f32
}

impl Normal {
    fn f(&self, x: f32) -> f32 {
        (1f32 / (2f32 * PI * self.deviation).sqrt()) * E.pow(- ((x - self.mean).powf(2f32) / (2f32 * (self.deviation).powf(2f32))))
    }
}