use std::rc::Weak;
use rendering::RenderContext;
use wgpu::util::DeviceExt;

use super::{line::LineList, Line, LineVertex};
use crate::View;

//TODO this object should take a view of an ECS
//where all it can see is Rect objects, but for now
pub struct RectPipeline {
    pipeline: wgpu::RenderPipeline,
    render_context_ptr: Weak<RenderContext>
}

pub struct LineVertices {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl RectPipeline {
    fn new(render_context_ptr: Weak<RenderContext>, camera_layout: &wgpu::BindGroupLayout) -> Self {
        let bind_group_layouts = &[camera_layout];

        //upgrade the pointer here for the duration of new
        let render_context = render_context_ptr.upgrade().expect("Passed a deconstructed render context to render pipeline");

        let shader = render_context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("RectShader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("line.wgsl").into()),
            });

        let render_pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Rect Render Pipeline Layout"),
                    bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let pipeline =
            render_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Rect Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",         // 1.
                        buffers: &[LineVertex::desc()], // 2.
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
        Self {
            pipeline,
            render_context_ptr
        }
    }

    fn generate_buffers(&self, lines: &[Line], view: &View) -> LineVertices {
        //upgrade render context here
        let render_context = self.render_context_ptr.upgrade().expect("Can't generate line vertice buffers on a freed render context");
        
        let lines = LineList::_construct_from_vec(lines, view);

        let vertices: &[LineVertex] = lines.vertices();
        let indices: &[u16] = lines.indices();

        //in with the new
        let vertex_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
        let num_indices = indices.len() as u32;

        LineVertices {
            vertex_buffer,
            index_buffer,
            num_indices
        }
    }

    //take in a vertex buffer that has been created from this object
    fn render(&self, lines: LineVertices) {
        //again upgrade our render context
        let render_context = self.render_context_ptr.upgrade().expect("Can't render lines with an invalid context");

        
    }
}
