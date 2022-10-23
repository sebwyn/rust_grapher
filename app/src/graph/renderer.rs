use bevy_ecs::prelude::*;

use bevy_ecs::schedule::SingleThreadedExecutor;
use bevy_ecs::schedule::SystemStage;
use bevy_ecs::system::Commands;

use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

//use the 2d crate for this renderer
use rendering::RenderContext;
use rendering::Renderer;
use two_dimensional::{
    primitives::line::{CameraUniform, LineList, RectPipeline},
    CameraController, CameraMatrix, View,
};

use super::{equation::generate_equation_lines, grid_lines::generate_grid_lines};

//TODO: creating future renderers will be simpler if i abstract out the idea of a uniform
//idea for a point renderer, render a square and then turn it into a circle in the fragment shader
//also give an option of whether to render points as circles or squares

//need to think about the separation of renderer and camera object
//for example the current thought process, is that we should be able to construct
//a matrix from a view object, which means we can update camera whenever and then
//render with a view, or update a renderer whenever the view updates
//along with all the other objects that need to adapt to a changing view
pub struct GraphRenderContext {
    pub background_color: wgpu::Color,
}

pub struct RenderPassContext {
    pub view: View,
    pub lines: LineList,
}

pub fn init_graph_render_context(
    commands: Commands,
    window: Res<Window>,
    render_context: Res<RenderContext>,
) {
    let cam_controller = CameraController::new(0f32, 0f32, window.inner_size());
    //create a render context
    commands.insert_resource(cam_controller);

    //create our camera uniform here
    let camera_uniform = Self::construct_camera_uniform(&render_context, &cam_controller);
    //add our camera uniform as a resource
    commands.insert_resource(camera_uniform);

    //create the render_pipeline here
    let line_pipeline = RectPipeline::new(&render_context, &camera_uniform.bind_group_layout);
    commands.insert_resource(line_pipeline);

    let background_color = wgpu::Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    let graph_render_context = Self { background_color };
    commands.insert_resource(graph_render_context);
}

fn construct_camera_uniform(
    render_context: &RenderContext,
    cam_controller: &CameraController,
) -> CameraUniform {
    let camera_matrix: CameraMatrix = cam_controller.clone().into();

    let buffer = render_context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    let bind_group_layout =
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

    let bind_group = render_context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

    CameraUniform {
        bind_group_layout,
        bind_group,
        buffer,
    }
}


impl GraphRenderContext {
    //TODO: support multiple renderers by passing a reference to a render context
    //pass a camera object here that we can use and update outside of the renderer
    //but that we construct a renderer with so we can construct from that view

    //this is constructing a camera uniform

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

pub fn generate_render_stage() -> SystemStage {
    let render_stage = SystemStage::new(Box::new(SingleThreadedExecutor::default()))
        .add_system(init_render_pass)
        .add_system(
            generate_grid_lines
                .chain(generate_equation_lines)
                .chain(render_lines),
        )
        .add_system(end_render_pass);

    render_stage
}

fn begin_render(commands: Commands, render_context: Res<RenderContext>) {
    //get a view of the current window texture
    let output = render_context.surface.get_current_texture();
    commands.insert_resource(
        output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );

    //initialize our command buffers to zero
    let command_buffers: Vec<CommandBuffer> = Vec::new();
    commands.insert_resource(command_buffers);
}

fn begin_line_pass(commands: Commands) {
    //generate RenderPassData here
    RenderPassData {}
}

fn render_lines(context: ResMut<RenderContext>, render_pass_context: ResMut<RenderContext>) {}

impl Renderer for GraphRenderContext {
    fn render(&self) -> Result<(), wgpu::SurfaceError> {
        //render using our pipeline
        let output = render_context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        output.present();
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
            self.render_context.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[camera_matrix]),
            );

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
