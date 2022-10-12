use super::equation::Equation;

pub struct Quadratic;

impl Equation for Quadratic {
    fn f(&self, x: f32) -> f32 {
        x.powf(2.0f32)
    }
}