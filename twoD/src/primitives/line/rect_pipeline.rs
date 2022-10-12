use crate::render_context::RenderContext;

use super::{Vertex, Line, line::LineList};

//TODO this object should take a view of an ECS
//where all it can see is Rect objects, but for now 
pub struct RectPipeline {
    pipeline: wgpu::RenderPipeline,
}

pub struct LineVertices {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    num_indices: u32
}

impl RectPipeline {
    fn new(render_context: &RenderContext, camera_layout: &wgpu::BindGroupLayout) -> Self {
        let bind_group_layouts = &[camera_layout];
        
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

        let render_pipeline =
            render_context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Rect Render Pipeline"),
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
        Self {
            pipeline: render_pipeline
        }
    }


    fn generate_buffers(lines: &[Line], view: &View) -> LineVertices {
        let lines = LineList::_construct_from_vec(lines, view);
    }

    //take in a vertex buffer that has been created from this object
    fn render(lines: LineVertices) {

    }
}

