use super::{renderable::Renderable, line::{LineList, Line}, view::View};

struct VerticalLine;

impl VerticalLine {
    fn new(x: f32, width: f32, color: [f32; 3]) -> Line {
        Line {width, start: (x, -10000f32), end: (x, 10000f32), color}
    }
}

struct HorizontalLine;

impl HorizontalLine {
    fn new(y: f32, width: f32, color: [f32; 3]) -> Line {
        Line {width, start: (-10000f32, y), end: (10000f32, y), color}
    }
}

//gridlines needs to be able to generate a list of lines on the fly
//it stores the lines in generated before and only updates when the view changes
//it needs to take a view object 
pub struct GridLines {
    lines: LineList
}

impl GridLines {
    pub fn new() -> Self {
        Self {
            lines: LineList::new()
        }
    }
}

impl Renderable for GridLines {
    fn update(&mut self, view: &View) {
        self.lines = LineList::new();

        let incs_to_top = 5f32;
        let step = 2f32.powf(f32::trunc(f32::log2((view.top - view.center_y) / incs_to_top)));
        //get the starting x
        let mut x = f32::trunc(view.left / step) * step;
        while x < view.right {
            self.lines.add_line(VerticalLine::new(x, 5f32 / view.scale, [0.3, 0.3, 0.3]));
            x += step;
        }

        let mut y = f32::trunc(view.bottom / step) * step;
        while y < view.top {
            self.lines.add_line(HorizontalLine::new(y, 5f32 / view.scale, [0.3, 0.3, 0.3]));
            y += step;
        }

        self.lines.add_line(VerticalLine::new(0f32, 5f32 / view.scale, [0f32, 0f32, 0f32]));
        self.lines.add_line(HorizontalLine::new(0f32, 5f32 / view.scale, [0f32, 0f32, 0f32]));
        self.lines.add_line(Line {width: 5f32 / view.scale, start: (0f32, 0f32), end: (64f32, 64f32), color: [1f32, 0f32, 0f32]});
    }

    fn get_lines(&self) -> LineList {
        self.lines.clone()
    }
}