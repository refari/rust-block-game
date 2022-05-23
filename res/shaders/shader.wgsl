// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>;
};

[[group(1), binding(0)]] // bind
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] texcoord: vec2<f32>;
    [[location(2)]] normal: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] texcoord: vec2<f32>;
    [[location(1)]] normal: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var v_out: VertexOutput;
    v_out.texcoord = model.texcoord;
    v_out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    v_out.normal = model.normal;
    return v_out;
}

// Fragment shader

[[group(0), binding(0)]]
var tex: texture_2d<f32>;
[[group(0), binding(1)]]
var sam: sampler;

[[stage(fragment)]]
fn fs_main(v_in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var tex: vec4<f32> = textureSample(tex, sam, v_in.texcoord);

    tex = vec4<f32>(tex.rgb * (dot(normalize(v_in.normal), vec3<f32>(0.5, 0.75, 0.5))*0.25 + 0.75), 1.0);
    return tex;
}