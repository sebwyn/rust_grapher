use super::{line::Line, view::View};

pub trait Renderable {
    fn update(&mut self, view: &View);
    fn get_lines(&self) -> &Vec<Line>;
}