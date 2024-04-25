// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};


const points: array<vec4<f32>, 3> = array(
    vec4<f32>(-1.0, -3.0, 0.0, 1.0), // Bottom-left
    vec4<f32>(3.0, 1.0, 0.0, 1.0), // Right corner
    vec4<f32>(-1.0, 1.0, 0.0, 1.0) // Top-left
);

@vertex
fn vs_main(
    @builtin(vertex_index) idx: u32,
) -> VertexOutput {
    var out: VertexOutput;

    if idx == 0 {
        out.clip_position = vec4<f32>(-1.0, -3.0, 0.0, 1.0); // Bottom-left
    } else if idx == 1 {
        out.clip_position = vec4<f32>(3.0, 1.0, 0.0, 1.0); // Right corner

    } else if idx == 2 {
        out.clip_position = vec4<f32>(-1.0, 1.0, 0.0, 1.0); // Top-left
    }

	return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // TODO: render from a texture
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
