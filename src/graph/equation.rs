use super::{Renderable, line::Line, view::View};

pub trait Equation {
    fn f(&self, x: f32) -> f32;
}

//TODO: this sampling/scaling method does not work well for some functions
//particularly periodic functions like sin
impl<T> Renderable for T 
where
    T: Equation
{
    fn generate_lines(&self, view: &View) -> Vec<Line>
    {
        let mut lines = Vec::new();

        //in this case, we want 5 pixels per increment (5 pixel long lines)
        let line_length = 5f32;
        //TODO: do this in a rust idiomatic way
        //calculate what our x-step should be based on a 5 pixel increment
        let x_step = line_length / view.scale;
        let mut x = view.left;
        let mut y = self.f(x);
        while x < view.right {
            let next_x = x + x_step;
            let next_y = self.f(next_x);
            //draw a line from the two points
            lines.push(Line { width: 4f32, start: (x, y), end: (next_x, next_y), color: [1f32, 0f32, 0f32]});

            x = next_x;
            y = next_y;
        }

        lines
    }
}