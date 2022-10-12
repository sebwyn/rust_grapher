use std::{rc::Rc, cell::RefCell};

use wgpu::util::DeviceExt;
use winit::{window::Window, event::WindowEvent};

//use the 2d crate for this renderer
use rendering::RenderContext;
use rendering::Renderer;
use two_dimensional::{primitives::{Line, LineVertex}, View, CameraController};

use super::GraphRenderable;

//TODO: creating future renderers will be simpler if i abstract out the idea of a uniform
//idea for a point renderer, render a square and then turn it into a circle in the fragment shader
//also give an option of whether to render points as circles or squares

//need to think about the separation of renderer and camera object
//for example the current thought process, is that we should be able to construct 
//a matrix from a view object, which means we can update camera whenever and then
//render with a view, or update a renderer whenever the view updates
//along with all the other objects that need to adapt to a changing view
pub struct GraphRenderer {
    render_context: RenderContext,
    render_pipeline: wgpu::RenderPipeline,
    background_color: wgpu::Color,

    //our camera information
    cam_controller: CameraController,
    view_changed: bool,
    //this is camera uniform information
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    //early ECS concept
    renderables: Rc<RefCell<Vec<Box<dyn GraphRenderable>>>>,

    //trying to only update renderables when we need to (probably have some kind of edited flag later)
    renderables_len: usize,

    //cached vertex and index buffers
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    num_indices: u32
}

impl GraphRenderer {
    //TODO: support multiple renderers by passing a reference to a render context
    //pass a camera object here that we can use and update outside of the renderer
    //but that we construct a renderer with so we can construct from that view
    pub async fn new(window: &Window, renderables: Rc<RefCell<Vec<Box<dyn Renderable>>>>) -> Self {

        let cam_controller = CameraController::new(0f32, 0f32, window.inner_size());
        //create a render context
        let render_context = RenderContext::new(window).await;

        //create our camera uniform here
        let (camera_buffer, camera_bind_group_layout, camera_bind_group) =
            Self::construct_camera_uniform(&render_context, &cam_controller);

        //create the render_pipeline here
        let render_pipeline =
            Self::construct_render_pipeline(&render_context, &[&camera_bind_group_layout]);
        
        //dont initialize buffers here, because they will only be destroyed
        //instead before we render make sure there are actually vertices to render

        let background_color = wgpu::Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };

        Self {
            render_context,
            render_pipeline,
            background_color,

            cam_controller,
            view_changed: true, //update objects before rendering

            camera_buffer,
            camera_bind_group,
            
            renderables,
            renderables_len: 0,

            vertex_buffer: None,
            index_buffer: None,
            num_indices: 0
        }
    }

    //this is constructing a camera uniform
    fn construct_camera_uniform(
        render_context: &RenderContext,
        cam_controller: &CameraController
    ) -> (
        wgpu::Buffer,
        wgpu::BindGroupLayout,
        wgpu::BindGroup,
    ) {
        let camera_matrix: CameraMatrix = cam_controller.clone().into();

        let camera_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_matrix]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });

        let camera_bind_group =
            render_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &camera_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }],
                    label: Some("camera_bind_group"),
                });

        (
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        )
    }

    fn update_buffers(&mut self) {
        //iterate over renderables and construct a line list
        //lots of copying here very very expensive
        let view: View = self.cam_controller.clone().into();
        let mut lines = LineList::new();
        for ll in self.renderables.borrow().iter() {
            let next_lines = ll.generate_lines(&view);
            lines.append_vec(&next_lines, &view);
        }

    }
}

impl Renderer for GraphRenderer {
    fn render(&self) -> Result<(), wgpu::SurfaceError> {
        if let Some(vertex_buffer) = &self.vertex_buffer {
        if let Some(index_buffer) = &self.index_buffer {
            //render using our pipeline
            let output = self.render_context.surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder =
                self.render_context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.background_color),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }

            self.render_context
                .queue
                .submit(std::iter::once(encoder.finish()));

            output.present();
        }}
        Ok(())
    }

    //resize may be called with nothing signalling that we should reconfigure our render context
    fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
        //passing resize events to the render context
        if let Some(size) = new_size {
            self.render_context.resize(size);
            //resize our camera
            self.cam_controller.resize(size);
            self.view_changed = true;
        } else {
            self.render_context.resize(self.render_context.size);
        }

        //send resize events to registered renderable components with a camera view
    }

    fn update(&mut self) {
        //update our camera if the view has changed since the last update
        let new_renderable_len: usize = self.renderables.borrow().len();
        if self.view_changed {
            let camera_matrix: CameraMatrix = self.cam_controller.clone().into();
            self.render_context.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_matrix]));

            self.update_buffers();
        } else if new_renderable_len != self.renderables_len {
            //update everything because we don't know whats new
            self.update_buffers();
            self.renderables_len = new_renderable_len;
        }
    }

    //pass events to our cam controller
    fn event(&mut self, event: &WindowEvent) {
        self.view_changed = self.cam_controller.event(event);
    }
}
