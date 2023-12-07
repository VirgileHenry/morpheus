// Vertex shader

struct Vertex {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<u32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Cube vertices with integrated triangles - positions
    var quad_positions: array<vec4<f32>, 6> = array<vec4<f32>, 6> (
        vec4(-1.0, 1.0, 0.0, 1.0),
        vec4(-1.0, -1.0, 0.0, 1.0),
        vec4(1.0, 1.0, 0.0, 1.0),
        vec4(-1.0, -1.0, 0.0, 1.0),
        vec4(1.0, -1.0, 0.0, 1.0),
        vec4(1.0, 1.0, 0.0, 1.0),
    );

    return quad_positions[in_vertex_index];
}

struct ScreenResolution {
    width: u32,
    height: u32,
}

@group(0) @binding(0)
var<uniform> screen_resolution: ScreenResolution;

// frag shader

@group(1) @binding(0)
var gbuff_albedo_t: texture_2d<f32>;

@group(2) @binding(0)
var gbuff_normal_depth_t: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {

    let uv: vec2<u32> = vec2(
        u32(in.x),
        u32(in.y),
    );

    let ambiant = vec3(0.01, 0.01, 0.03);
    let sun = vec3(0.98, 0.95, 0.93);
    let sun_dir = normalize(vec3(-0.3, 1.0, -0.4));

    let normal_depth = textureLoad(gbuff_normal_depth_t, uv, 0);
    let color = textureLoad(gbuff_albedo_t, uv, 0).xyz;

    let normal = normal_depth.xyz;
    let depth = normal_depth.w;
    
    let light = max(0.0, dot(-sun_dir, normal)) * sun + ambiant;

    let shaded = color * light;
    return vec4(shaded, 1.0);
}
