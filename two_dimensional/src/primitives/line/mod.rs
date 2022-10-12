mod vertex;
mod line;
mod rect_pipeline;

//expose our lines and vertices to the pipeline at least
pub use vertex::Vertex as LineVertex;
pub use line::Line;
pub use rect_pipeline::RectPipeline;
