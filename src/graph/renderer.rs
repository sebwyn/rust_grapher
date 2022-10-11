use std::{rc::Rc, cell::RefCell};

use wgpu::util::DeviceExt;
use winit::{window::Window, event::WindowEvent};

use super::{
    camera::{CameraController, CameraMatrix},
    vertex::Vertex, line::{LineList}, renderable::Renderable, view::View
};
use crate::{render_context::RenderContext, renderer::Renderer};

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
    renderables: Rc<RefCell<Vec<Box<dyn Renderable>>>>,

    //trying to only update renderables when we need to (probably have some kind of edited flag later)
    renderables_len: usize,

    //cached vertex and index buffers
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
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
        
        let vertices: Vec<Vertex> = Vec::new();
        let indices: Vec<u16> = Vec::new();
        println!("{:?}", vertices);

        //TODO remove this test code
        let vertex_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(indices.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                });
        let num_indices = indices.len() as u32;

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

            //camera info
            cam_controller,
            view_changed: true, //update objects before rendering

            camera_buffer,
            camera_bind_group,
            
            //ECS
            renderables,

            renderables_len: 0,

            vertex_buffer,
            index_buffer,
            num_indices
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

    fn construct_render_pipeline(
        render_context: &RenderContext,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let shader = render_context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("line.wgsl").into()),
            });

        let render_pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            render_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",     // 1.
                        buffers: &[Vertex::desc()], // 2.
                    },
                    fragment: Some(wgpu::FragmentState {
                        // 3.
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            // 4.
                            format: render_context.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw, // 2.
                        cull_mode: None,                  //Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: None, // 1.
                    multisample: wgpu::MultisampleState {
                        count: 1,                         // 2.
                        mask: !0,                         // 3.
                        alpha_to_coverage_enabled: false, // 4.
                    },
                    multiview: None, // 5.
                });

        render_pipeline
    }

    fn update_buffers(&mut self) {
        //iterate over renderables and construct a line list
        //lots of copying here very very expensive
        let view: View = self.cam_controller.clone().into();
        let mut lines = LineList::new();
        for ll in self.renderables.borrow().iter() {
            let next_lines = ll.get_lines();
            lines.append_vec(next_lines, &view);
        }

        let vertices: Vec<Vertex> = lines.vertices;
        let indices: Vec<u16> = lines.indices;

        //TODO remove this test code
        self.vertex_buffer =
            self.render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        self.index_buffer =
            self.render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(indices.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                });
        self.num_indices = indices.len() as u32;
    }
}

impl Renderer for GraphRenderer {
    fn render(&self) -> Result<(), wgpu::SurfaceError> {
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
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.render_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        output.present();

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
        if self.view_changed {
            let camera_matrix: CameraMatrix = self.cam_controller.clone().into();
            self.render_context.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_matrix]));

            let view = self.cam_controller.clone().into();
            //pass the updated view to our renderables and reconstruct
            for renderable in self.renderables.borrow_mut().iter_mut() {
                renderable.update(&view);
            }
            self.update_buffers();
        } else if self.renderables.borrow().len() != self.renderables_len {
            self.update_buffers();
        }
    }

    //pass events to our cam controller
    fn event(&mut self, event: &WindowEvent) {
        self.view_changed = self.cam_controller.event(event);
    }
}
