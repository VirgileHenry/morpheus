// Vertex shader

struct Camera {
    proj_view: mat4x4<f32>,
    inv_view: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) screen_position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {

    // todo : I hate this. Any way to make does arrays global ?

    // Cube vertices with integrated triangles - positions
    var positions: array<vec4<f32>, 36> = array<vec4<f32>, 36>(
        /* Front face   */
        vec4<f32>(-0.5, 0.5, -0.5, 1.0), vec4<f32>(-0.5, -0.5, -0.5, 1.0), vec4<f32>(0.5, -0.5, -0.5, 1.0),
        vec4<f32>(-0.5, 0.5, -0.5, 1.0), vec4<f32>(0.5, -0.5, -0.5, 1.0), vec4<f32>(0.5, 0.5, -0.5, 1.0),

        /* Back face    */
        vec4<f32>(-0.5, 0.5, 0.5, 1.0), vec4<f32>(-0.5, -0.5, 0.5, 1.0), vec4<f32>(0.5, -0.5, 0.5, 1.0),
        vec4<f32>(-0.5, 0.5, 0.5, 1.0), vec4<f32>(0.5, -0.5, 0.5, 1.0), vec4<f32>(0.5, 0.5, 0.5, 1.0),

        /* Left face    */
        vec4<f32>(-0.5, 0.5, -0.5, 1.0), vec4<f32>(-0.5, -0.5, -0.5, 1.0), vec4<f32>(-0.5, -0.5, 0.5, 1.0),
        vec4<f32>(-0.5, 0.5, -0.5, 1.0), vec4<f32>(-0.5, -0.5, 0.5, 1.0), vec4<f32>(-0.5, 0.5, 0.5, 1.0),

        /* Right face   */
        vec4<f32>(0.5, 0.5, -0.5, 1.0), vec4<f32>(0.5, -0.5, -0.5, 1.0), vec4<f32>(0.5, -0.5, 0.5, 1.0),
        vec4<f32>(0.5, 0.5, -0.5, 1.0), vec4<f32>(0.5, -0.5, 0.5, 1.0), vec4<f32>(0.5, 0.5, 0.5, 1.0),

        /* Bottom face  */
        vec4<f32>(-0.5, 0.5, -0.5, 1.0), vec4<f32>(0.5, 0.5, -0.5, 1.0), vec4<f32>(0.5, 0.5, 0.5, 1.0),
        vec4<f32>(-0.5, 0.5, -0.5, 1.0), vec4<f32>(0.5, 0.5, 0.5, 1.0), vec4<f32>(-0.5, 0.5, 0.5, 1.0),

        /* Top face     */
        vec4<f32>(-0.5, -0.5, -0.5, 1.0), vec4<f32>(0.5, -0.5, -0.5, 1.0), vec4<f32>(0.5, -0.5, 0.5, 1.0),
        vec4<f32>(-0.5, -0.5, -0.5, 1.0), vec4<f32>(0.5, -0.5, 0.5, 1.0), vec4<f32>(-0.5, -0.5, 0.5, 1.0)
    );

    var out: VertexOutput;
    out.screen_position = camera.proj_view * positions[in_vertex_index];
    return out;
}

struct ScreenResolution {
    width: u32,
    height: u32,
}

@group(1) @binding(0)
var<uniform> screen_resolution: ScreenResolution;

struct CsgSphere { // size 16
    offset: vec3<f32>,
    radius: f32,
}

struct CsgObject { // size 32
    csg_id: u32,
    /// the data can be interpreted as any values depending on the id.
    /// It's a hand made union
    data: array<f32, 7>,
}

@group(2) @binding(0)
var<storage> csg_objects: array<CsgObject>;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sphere = csg_objects[0];
    let sphere_id = sphere.csg_id;
    // it's suppose to be 10
    let color = in.screen_position;

    return color;
}

struct Ray {
    origin: vec3<f32>,
    dir: vec3<f32>,
}


fn getRayFromClipSpace(fragPos: vec4<f32>, invProjectionMatrix: mat4x4<f32>) -> vec3<f32> {
    // Homogeneous coordinates
    let clipSpaceRay = vec4<f32>(fragPos.x, fragPos.y, -1.0, 1.0);

    // Invert the projection matrix
    let worldSpaceRay = invProjectionMatrix * clipSpaceRay;

    // Normalize the resulting vector to get a direction
    let rayDirection = normalize(worldSpaceRay.xyz);

    return rayDirection;
}

fn scene_sdf(at: vec3<f32>) -> f32 {
    // this index will be passed to sdf as they parse the tree.
    var csg_index: u32 = 0u;
    switch csg_objects[csg_index].csg_id {
        case 0u: { return 1.0 / 0.0; } // 0 is empty object, return inf
        case 1u: { return sphere_sdf(at, &csg_index); } // 1 is sphere


        default: { return 0.; } // csg obj not supported, stop
    }
}

/// Get the sdf of a sphere,
/// considering the sphere data is in the csg_objects array at the given index.
fn sphere_sdf(at: vec3<f32>, csg_index: ptr<function, u32>) -> f32 {
    // we are reading this primitive, so increase the index
    let data: array<f32, 7> = csg_objects[*csg_index].data;
    let offset: vec3<f32> = vec3(data[0], data[1], data[2]);
    let radius = data[3];
    *csg_index += 1u;
    return length(offset - at) - radius;
}