mod vertex;
mod line;
mod rect_pipeline;

//expose our lines and vertices to the pipeline at least
pub use vertex::Vertex as LineVertex;
pub use line::Line;
pub use line::LineList;
pub use rect_pipeline::RectPipeline;
pub use rect_pipeline::render as render_lines;
pub use rect_pipeline::RenderPassData as LinePassData;
pub use rect_pipeline::CameraUniform;
