use wgpu::util::DeviceExt;
use winit::window::Window;

use super::{
    camera::{CameraController, CameraUniform},
    graph_vertex::Vertex, line::{Line, LineVertexListBuilder}
};
use crate::{render_context::RenderContext, renderer::Renderer};

//TODO: creating future renderers will be simpler if i abstract out the idea of a uniform
pub struct GraphRenderer {
    render_context: RenderContext,
    camera_controller: CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    background_color: wgpu::Color,
    //remove this code
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl GraphRenderer {
    //TODO: support multiple renderers by passing a reference to a render context
    pub async fn new(window: &Window) -> Self {
        //create a render context
        let render_context = RenderContext::new(window).await;

        //create our camera here
        let (camera_controller, camera_buffer, camera_bind_group_layout, camera_bind_group) =
            Self::create_camera_uniform(&render_context);

        //create the render_pipeline here
        let render_pipeline =
            Self::create_render_pipeline(&render_context, &[&camera_bind_group_layout]);

        //generate 2 lines here
        let vertex_list_builder = LineVertexListBuilder::new();
        let vertex_list = vertex_list_builder
            .add_line(Line {width: 0.5f32, start: (0f32, -10000f32), end: (0f32, 10000f32), color: [0f32, 0f32, 0f32]})
            .add_line(Line {width: 0.25f32, start: (0f32, 0f32), end: (10f32, 10f32), color: [0f32, 0f32, 0f32]})
            .add_line(Line {width: 0.5f32, start: (-10000f32, 0f32), end: (10000f32, 0f32), color: [0f32, 0f32, 0f32]});
        let vertices = vertex_list.vertices;
        let indices = vertex_list.indices;

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
            camera_controller,
            camera_buffer,
            camera_bind_group,
            render_pipeline,
            background_color,
            //remove these
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }

    fn create_camera_uniform(
        render_context: &RenderContext,
    ) -> (
        CameraController,
        wgpu::Buffer,
        wgpu::BindGroupLayout,
        wgpu::BindGroup,
    ) {
        let camera = CameraController::new(0f32, 0f32, render_context.size.into());
        let camera_uniform: CameraUniform = camera.clone().into();

        let camera_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
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
            camera,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        )
    }

    fn create_render_pipeline(
        render_context: &RenderContext,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let shader = render_context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("graph_line.wgsl").into()),
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

    fn update_camera_uniform(&mut self) {
        let camera_uniform: CameraUniform = self.camera_controller.clone().into();
        //println!("Camera Uniform: {:?}", camera_uniform.view_ortho);
        self.render_context.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
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

    fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
        //passing resize events to the render context
        if let Some(size) = new_size {
            self.render_context.resize(size);

            //pass resize events to the cam controller, and update our camera uniforms
            self.camera_controller.resize((size.width, size.height));
            self.update_camera_uniform();
            
        } else {
            self.render_context.resize(self.render_context.size);
        }
    }

    fn event(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            e => {
                let result = self.camera_controller.event(e);
                self.update_camera_uniform();
                result
            },
        }
    }
}
