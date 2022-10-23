use bevy_ecs::prelude::*;
use two_dimensional::{
    primitives::line::{Line, LineList},
    View,
};

pub trait Equation {
    fn f(&self, x: f32) -> f32;
}


#[derive(Component)]
pub struct EquationBox {
    equation: Box<dyn Equation + Send + Sync>
}

//implement a system that generates lines for all equations
pub fn generate_equation_lines(query: Query<&EquationBox>, view: Res<View>, lines: ResMut<LineList>) {
    for eq_box in &query {
        let equation = eq_box.equation;

        //in this case, we want 5 pixels per increment (5 pixel long lines)
        let line_width = 5f32;
        //TODO: do this in a rust idiomatic way
        //calculate what our x-step should be based on a 5 pixel increment
        let x_step = line_width / view.scale;
        let mut x = view.left;
        let mut y = equation.f(x);
        while x < view.right {
            let next_x = x + x_step;
            let next_y = equation.f(next_x);
            //draw a line from the two points
            lines.add_line(
                &Line {
                    width: 4f32,
                    start: (x, y),
                    end: (next_x, next_y),
                    color: [1f32, 0f32, 0f32],
                },
                view.as_ref(),
            );

            x = next_x;
            y = next_y;
        }
    }
}