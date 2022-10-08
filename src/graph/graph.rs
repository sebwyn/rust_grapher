use std::borrow::BorrowMut;

use bytemuck::bytes_of;
use wgpu::TextureView;
use winit::event::WindowEvent;
use wgpu::util::DeviceExt;

use super::graph_vertex::Vertex;
use super::graph_camera::{GraphCamera, GraphCameraUniform};

use crate::state::State;

pub struct Graph {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    bacground_color: wgpu::Color,
    //graph matrix for transforming from graph coordinates to camera space
    //maybe unnessecary because we could just have a camera matrix
    camera: GraphCamera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup
}

impl Graph {
    //construct our camera, construct our render pipeline
    //TODO: construct a line as a more complex polygon
    pub fn new(state: &State) -> Self {
        let camera = GraphCamera::new(0f32, 0f32, state.size.into());

        //create our camera uniform here
        let camera_uniform: GraphCameraUniform = camera.clone().into();

        println!("{:?}", camera.get_view_ortho_matrix() * cgmath::Vector4 { x: 0.0, y: 10.0, z: 1.0, w: 0.0 });
        
        let camera_buffer = state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        //create the render_pipeline here
        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("graph_line.wgsl").into()),
            });

        let render_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            state
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
                            format: state.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw, // 2.
                        cull_mode: None, //Some(wgpu::Face::Back),
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

        //generate a vertex buffer of lines here
        let mut vertices = Vec::new();
        vertices.push(Vertex {
            position: [-0.5, -10000.0, 0.0],
            color: [0.0, 0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [0.5, -10000.0, 0.0],
            color: [0.0, 0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [-0.5, 10000.0, 0.0],
            color: [0.0, 0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [0.5, 10000.0, 0.0],
            color: [0.0, 0.0, 0.0],
        });

        vertices.push(Vertex {
            position: [-10000.0, -0.5, 0.0],
            color: [0.0, 0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [-10000.0, 0.5, 0.0],
            color: [0.0, 0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [10000.0, -0.5, 0.0],
            color: [0.0, 0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [10000.0, 0.5, 0.0],
            color: [0.0, 0.0, 0.0],
        });

        let vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            });
        
        let indices: Vec<u16> = vec![0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7];

        let index_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;
        

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            bacground_color: wgpu::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            camera,
            camera_buffer,
            camera_bind_group
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        //update our graph and camera here
        /*self.camera.borrow_mut().resize(new_size.into());
        //update our camera uniform here
        state.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[GraphCameraUniform::from(self.camera)]));
        */
    }

    pub fn update(&mut self) {
        //update the graph here
    }

    //handle events
    pub fn event(&self, _window_event: &WindowEvent) {}

    //render using our pipeline
    pub fn render(&self, state: &State, view: &TextureView) -> wgpu::CommandEncoder {
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.bacground_color),
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

        encoder
    }
}
