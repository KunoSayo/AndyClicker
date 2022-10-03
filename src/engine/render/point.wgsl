struct VertexOutput {
    @location(0) c: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@location(0) a_color: vec4<f32>, @location(1) a_pos: vec2<f32>, @builtin(vertex_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;
    out.c = a_color;
    out.position = vec4<f32>(a_pos[0], a_pos[1], 0.5, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.c;
}
