// Vertex shader

struct Camera {
    proj_view: mat4x4<f32>,
    view_mat: mat4x4<f32>,
    position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;


@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {

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

    return camera.proj_view * positions[in_vertex_index];
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
    /// It's a hand made union thingy
    data: array<f32, 7>,
}

@group(2) @binding(0)
var<storage> csg_objects: array<CsgObject>;


@fragment
fn fs_main(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {
    
    let screen_pos = vec2(in.x / f32(screen_resolution.width) - 0.5, in.y / f32(screen_resolution.height) - 0.5);
    let ray: Ray = get_ray(screen_pos);
    // advance ray towards first box encounter

    let max_iter = 100;
    let hit_eps = 0.01;
    var eval_point: vec3<f32> = ray.origin;
    for(var i = 0; i < max_iter; i++) {
        let scene_sdf = scene_sdf(eval_point);
        if(scene_sdf < hit_eps) {
            // it's a hit ! 
            return vec4(1.0, 1.0, f32(i / max_iter), 1.0);
        }
        eval_point += ray.dir * scene_sdf;
    }

    // infinity, discard
    discard;
    
    // temp debug to see the bounding box
    // return vec4(1.0, 0.0, 0.0, 1.0);

}

struct Ray {
    origin: vec3<f32>,
    dir: vec3<f32>,
}

fn get_ray(noramalized_frag_pos: vec2<f32>) -> Ray {
    let cam_forward = vec4(0.0, 0.0, -1.0, 1.0) * camera.view_mat;
    let cam_right = vec4(1.0, 0.0, 0.0, 1.0) * camera.view_mat;
    let cam_up = vec4(0.0, 1.0, 0.0, 1.0) * camera.view_mat;
    let cam_fovy = 1.5; // todo : hard coded as cpu side for now
    let inv_aspect_ratio = f32(screen_resolution.height) / f32(screen_resolution.width);

    let x = noramalized_frag_pos.x;
    let y = noramalized_frag_pos.y * inv_aspect_ratio;
    let zoom = 0.5 / tan(cam_fovy * 0.5);

    // Normalize the resulting vector to get a direction
    let ray_direction = normalize((cam_forward * zoom + cam_right * x + cam_up * y).xyz);

    // todo: also convert the space with model mat 

    var ray: Ray;
    ray.origin = camera.position.xyz;
    ray.dir = ray_direction;

    return ray;
}

fn scene_sdf(at: vec3<f32>) -> f32 {
    // this index will be passed to sdf as they parse the tree.
    var csg_index = 0u;
    switch csg_objects[csg_index].csg_id {
        case 0u: { return 1.0 / 0.0; } // 0 is empty object, return inf
        case 1u: { return sphere_sdf(at, &csg_index); } // 1 is sphere

        default: { return 0.7; } // csg obj not supported, stop
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
    return length(at - offset) - radius;
}