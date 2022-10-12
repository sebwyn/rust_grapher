use super::{renderable::Renderable, line::Line, view::View};

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

//gridlines needs to be able to generate a list of lines on the fly
//it stores the lines in generated before and only updates when the view changes
//it needs to take a view object 
pub struct GridLines;

impl Renderable for GridLines {
    //TODO: make scale variable
    fn generate_lines(&self, view: &View) -> Vec<Line> {
        let mut lines = Vec::new();

        let incs_to_top = 5f32;
        let step = 2f32.powf(f32::trunc(f32::log2((view.top - view.center_y) / incs_to_top)));
        
        //construct all the vertical lines within view, with an step
        let mut x = f32::trunc(view.left / step) * step; //fancy line for finding the first x-pos
        while x < view.right {
            lines.push(VerticalLine::new(x, view.bottom, view.top, 2f32, [0.3, 0.3, 0.3]));
            x += step;
        }

        //construct all the horizontal lines within view, with a step
        let mut y = f32::trunc(view.bottom / step) * step;
        while y < view.top {
            lines.push(HorizontalLine::new(y, view.left, view.right, 2f32, [0.3, 0.3, 0.3]));
            y += step;
        }

        //construct our axis in a different color
        lines.push(VerticalLine::new(0f32, view.bottom, view.top, 4f32, [0f32, 0f32, 0f32]));
        lines.push(HorizontalLine::new(0f32, view.left, view.right, 4f32, [0f32, 0f32, 0f32]));

        lines
    }

}