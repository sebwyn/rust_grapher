use super::{line::LineList, view::View};

pub trait Renderable {
    fn update(&self, view: &View);
    fn get_lines(&self) -> LineList;
}