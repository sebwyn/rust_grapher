use super::{line::Line, view::View};

//for now most renderable implementations aren't going to cache anything
//but instead choose to regenerate every time they have too
pub trait Renderable {
    fn generate_lines<'a>(&self, lines: &'a mut Vec<Line>, view: &View) -> &'a Vec<Line>;
}