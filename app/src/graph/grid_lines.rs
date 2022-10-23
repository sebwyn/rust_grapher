use bevy_ecs::prelude::*;
use two_dimensional::{primitives::line::{Line, LineList, LinePassData}, View};

struct VerticalLine;
impl VerticalLine {
    fn new(x: f32, bottom: f32, top: f32, width: f32, color: [f32; 3]) -> Line {
        Line {width, start: (x, bottom), end: (x, top), color}
    }
}

struct HorizontalLine;
impl HorizontalLine {
    fn new(y: f32, left: f32, right: f32, width: f32, color: [f32; 3]) -> Line {
        Line {width, start: (left, y), end: (right, y), color}
    }
}

pub fn generate_grid_lines(In(line_pass_data): In<LinePassData>) -> LinePassData {
    let view: &View = &line_pass_data.view;
    let lines: &mut LineList = &mut line_pass_data.lines;

    let incs_to_top = 5f32;
    let step = 2f32.powf(f32::trunc(f32::log2((view.top - view.center_y) / incs_to_top)));
    
    //construct all the vertical lines within view, with an step
    let mut x = f32::trunc(view.left / step) * step; //fancy line for finding the first x-pos
    while x < view.right {
        lines.add_line(&VerticalLine::new(x, view.bottom, view.top, 2f32, [0.3, 0.3, 0.3]), view);
        x += step;
    }

    //construct all the horizontal lines within view, with a step
    let mut y = f32::trunc(view.bottom / step) * step;
    while y < view.top {
        lines.add_line(&HorizontalLine::new(y, view.left, view.right, 2f32, [0.3, 0.3, 0.3]), view);
        y += step;
    }

    //construct our axis in a different color
    lines.add_line(&VerticalLine::new(0f32, view.bottom, view.top, 4f32, [0f32, 0f32, 0f32]), view);
    lines.add_line(&HorizontalLine::new(0f32, view.left, view.right, 4f32, [0f32, 0f32, 0f32]), view);

    line_pass_data
}