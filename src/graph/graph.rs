use wgpu::TextureView;
use winit::event::WindowEvent;

use crate::state::State;
use super::graph_vertex::Vertex;

use wgpu::util::DeviceExt;

pub struct Graph {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    //index_buffer: wgpu::Buffer,
    //num_indices: u32,
    num_vertices: u32,
    bacground_color: wgpu::Color
}

impl Graph {
    //construct our camera, construct our render pipeline
    //TODO: construct a line as a more complex polygon
    pub fn new(state: &State) -> Self {
        //create the render_pipeline here
        let shader = state.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("graph_line.wgsl").into()),
        });

        let render_pipeline_layout =
            state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                topology: wgpu::PrimitiveTopology::LineList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,
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
        vertices.push(Vertex { position: [ 0.0, -1.0, 0.0], color: [0.0, 0.0, 0.0]});
        vertices.push(Vertex { position: [ 0.0,  1.0, 0.0], color: [0.0, 0.0, 0.0]});
        vertices.push(Vertex { position: [-1.0,  0.0, 0.0], color: [0.0, 0.0, 0.0]});
        vertices.push(Vertex { position: [ 1.0,  0.0, 0.0], color: [0.0, 0.0, 0.0]});

        let vertex_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_vertices = vertices.len() as u32;
        /*
        let indices = vec![0, 1, 2, 3];

        let index_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;
        */

        Self {
            render_pipeline,
            vertex_buffer,
            num_vertices,
            bacground_color: wgpu::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
        }
    }

    pub fn update(&mut self) {
        //update the graph here

    }

    //handle events
    pub fn event(&self, _window_event: &WindowEvent) {

    }

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
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            //render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        encoder
    }
}