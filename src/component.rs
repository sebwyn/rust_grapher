pub trait Component {
    fn update(&mut self);
    fn event(&self, event: &winit::event::WindowEvent);
    fn resize(&self, new_size: &winit::dpi::PhysicalSize<u32>);
}