pub trait Renderer {
    fn render(&self) -> Result<(), wgpu::SurfaceError>;
    fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>);
    fn event(&self, event: &winit::event::WindowEvent) -> bool;
}