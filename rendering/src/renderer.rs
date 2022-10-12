use winit::event::WindowEvent;

pub trait Renderer {
    fn render(&self) -> Result<(), wgpu::SurfaceError>;
    fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>);
    fn update(&mut self);
    fn event(&mut self, event: &WindowEvent);
}