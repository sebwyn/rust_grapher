use bevy_ecs::prelude::*;

use rendering::RenderContext;
use wgpu::util::DeviceExt;

use super::{line::LineList, LineVertex};
use crate::View;

//TODO this object should take a view of an ECS
//where all it can see is Rect objects, but for now
pub struct RectPipeline {
    pipeline: wgpu::RenderPipeline,
}

pub struct CameraUniform {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

pub struct RenderPassData {
    pub view: View,
    pub lines: LineList,
}

fn generate_buffers(
    render_context: &RenderContext,
    lines: LineList,
) -> (wgpu::Buffer, wgpu::Buffer, u32) {
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

    (vertex_buffer, index_buffer, num_indices)
}

//define a bevy render system
pub fn render(
    In(render_pass_data): In<RenderPassData>,
    rect_pipeline: Res<RectPipeline>,
    render_context: Res<RenderContext>,
    surface_view: Res<wgpu::TextureView>,
    camera_uniform: Res<CameraUniform>,
    mut command_buffers: ResMut<Vec<wgpu::CommandBuffer>>,
) {
    //generate our vertex and index buffers here from our vertex data and index data
    let (vertex_buffer, index_buffer, num_indices) =
        generate_buffers(&render_context, render_pass_data.lines);

    let mut encoder =
        render_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Line Command Encoder"),
            });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&rect_pipeline.pipeline);
        render_pass.set_bind_group(0, &camera_uniform.bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..num_indices, 0, 0..1);
    }

    command_buffers.as_mut().push(encoder.finish());
}

impl RectPipeline {
    pub fn new(render_context: &RenderContext, camera_layout: &wgpu::BindGroupLayout) -> Self {
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
        Self { pipeline }
    }
}
