use super::{renderable::Renderable, line::{LineList, Line}, view::View};

//gridlines needs to be able to generate a list of lines on the fly
//it stores the lines in generated before and only updates when the view changes
//it needs to take a view object 
pub struct GridLines {
    lines: LineList
}

impl GridLines {
    pub fn new() -> Self {
        let mut lines = LineList::new();
        lines.add_line(Line {width: 0.5f32, start: (0f32, -10000f32), end: (0f32, 10000f32), color: [0f32, 0f32, 0f32]});
        lines.add_line(Line {width: 0.5f32, start: (-10000f32, 0f32), end: (10000f32, 0f32), color: [0f32, 0f32, 0f32]});
        lines.add_line(Line {width: 0.25f32, start: (0f32, 0f32), end: (10f32, 10f32), color: [1f32, 0f32, 0f32]});

        Self {
            lines
        }
    }
}

impl Renderable for GridLines {
    fn update(&self, view: &View) {
        //for now dont do anything with the new view
    }

    fn get_lines(&self) -> LineList {
        self.lines.clone()
    }
}