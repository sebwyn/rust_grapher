struct VertexInput {
    @location(0) position: vec3<f32>
}

struct VertexOutput2 {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput
) -> VertexOutput2 {
    var out: VertexOutput2;
    //nifty code to generate a triangle
    out.clip_position = vec4<f32>(vertex.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput2) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}