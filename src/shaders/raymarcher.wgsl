// Vertex shader

struct Camera {
    proj_view: mat4x4<f32>,
    ray_mat: mat4x4<f32>,
    position: vec3<f32>,
    fovy: f32,
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

// frag shader


struct ScreenResolution {
    width: u32,
    height: u32,
}

@group(1) @binding(0)
var<uniform> screen_resolution: ScreenResolution;

struct CsgObject {
    csg_id: u32,
    data: array<f32, 11>, // hand made union thingy
}

struct Ray {
    origin: vec3<f32>,
    dir: vec3<f32>,
}

struct GBufferOut {
    @location(0) albedo: vec4<f32>,
    @location(1) normal_depth: vec4<f32>,
}

@group(2) @binding(0)
var<storage> csg_objects: array<CsgObject>;
@group(2) @binding(1)
var<uniform> csg_object_count: u32;

/// noramalized_frag_pos should be between -1 and 1
fn get_ray(noramalized_frag_pos: vec2<f32>) -> Ray {
    let cam_right = (vec4(1.0, 0.0, 0.0, 1.0) * camera.ray_mat).xyz;
    let cam_up = (vec4(0.0, 1.0, 0.0, 1.0) * camera.ray_mat).xyz;
    let cam_forward = (vec4(0.0, 0.0, -1.0, 1.0) * camera.ray_mat).xyz;

    let aspect_ratio = f32(screen_resolution.width) / f32(screen_resolution.height);

    let tan_cam_fovy_halfed = tan(camera.fovy * 0.5);
    let tan_cam_fovx_halfed = aspect_ratio * tan_cam_fovy_halfed;

    let x = noramalized_frag_pos.x * tan_cam_fovx_halfed;
    let y = noramalized_frag_pos.y * tan_cam_fovy_halfed;

    let ray_direction = normalize(cam_forward + cam_right * x + cam_up * y);

    // todo: also convert the space with model mat 

    return Ray(camera.position, ray_direction);
}

@fragment
fn fs_main(@builtin(position) in: vec4<f32>) -> GBufferOut {
    
    let screen_pos = vec2(
        (in.x / f32(screen_resolution.width) - 0.5) * 2.0,
        // y is inverted because up is +y, but on screen y goes down
        (0.5 - in.y / f32(screen_resolution.height)) * 2.0,
    );
    let ray: Ray = get_ray(screen_pos);

    // if(true) { return GBufferOut(vec4(-ray.dir, 1.0), vec4(1.0)); }

    // todo : advance ray towards first box encounter
    // todo : stop condition: out of the box


    let max_iter = 250;
    let hit_eps = 0.00001;
    var eval_point: vec3<f32> = ray.origin;
    for(var i = 0; i < max_iter; i++) {
        let scene_sdf = scene_sdf(eval_point);
        if(scene_sdf < hit_eps) {
            // it's a hit !
            let albedo: vec4<f32> = vec4(1.0); // todo : materials
            let depth = length(eval_point - camera.position);
            let normal_depth: vec4<f32> = vec4(scene_normal(eval_point), depth);
            return GBufferOut(albedo, normal_depth);
        }
        eval_point += ray.dir * scene_sdf;
    }

    // if(true) { return GBufferOut(vec4(1.0, 0.0 ,0.0, 1.0), vec4(1.0)); }

    // infinity, discard
    discard;

}

fn scene_sdf(at: vec3<f32>) -> f32 {
    // the csg tree is written in reverse polish notation (suffixed)
    // use a stack to compute the sdf
    var stack_ptr: u32 = 0u;
    // hard coded stack size
    var sdf_stack: array<f32, 5>;

    for(var i: u32 = 0u; i < csg_object_count; i++) {

        switch csg_objects[i].csg_id {
            case 0u: { return 1.0 / 0.0; } // 0 is empty object, return inf
            case 1u: { // id 1 is sphere, push it on the stack
                sdf_stack[stack_ptr] = sphere_sdf(at, i);
                stack_ptr += 1u;
            }
            case 2u: { // id 2 is cube, push it on the stack
                sdf_stack[stack_ptr] = cube_sdf(at, i);
                stack_ptr += 1u;
            }

            case 22u: { // id 22 is union (min), from the two values on the stack
                let sdf1: f32 = sdf_stack[stack_ptr - 2u];
                let sdf2: f32 = sdf_stack[stack_ptr - 1u];
                sdf_stack[stack_ptr - 2u] = min(sdf1, sdf2);
                stack_ptr -= 1u; // pop 2 push 1
            }

            default: { return 0.; } // csg obj not supported, stop
        }
    }

    // the final result is last stack value !
    return sdf_stack[stack_ptr - 1u];
}


fn scene_normal(at: vec3<f32>) -> vec3<f32> {
    // mmmh, not a fan of calculating the sdf 4 times
    // another solution is to come across exact normal for every sdf node,
    // and use another stack to compute it

    // this comes from inigo quilez articles, and he said he does it 
    // https://iquilezles.org/articles/normalsSDF/
    
    // small enough for graphic precision, yet big enough to avoid noise artifacts
    let h: f32 = 0.00001;
    let k: vec2<f32> = vec2(1.0, -1.0);
    return normalize( k.xyy * scene_sdf( at + k.xyy * h ) + 
                      k.yyx * scene_sdf( at + k.yyx * h ) + 
                      k.yxy * scene_sdf( at + k.yxy * h ) + 
                      k.xxx * scene_sdf( at + k.xxx * h ) );
}


// all objects sdf


fn sphere_sdf(at: vec3<f32>, csg_index: u32) -> f32 {
    // we are reading this primitive, so increase the index
    let data: array<f32, 11> = csg_objects[csg_index].data;
    let offset: vec3<f32> = vec3(data[0], data[1], data[2]);
    let radius = data[3];
    return length(offset - at) - radius;
}

fn sphere_normal(at: vec3<f32>, csg_index: u32) -> vec3<f32> {
    let data: array<f32, 11> = csg_objects[csg_index].data;
    let center: vec3<f32> = vec3(data[0], data[1], data[2]);
    return normalize(at - center);
}

fn cube_sdf(at: vec3<f32>, csg_index: u32) -> f32 {
    let data: array<f32, 11> = csg_objects[csg_index].data;
    let position: vec3<f32> = vec3(data[0], data[1], data[2]);
    let rotation: vec4<f32> = vec4(data[3], data[4], data[5], data[6]);
    let scale: vec3<f32> = vec3(data[7], data[8], data[9]);
    let aligned = at - position;
    let rotated = aligned; // todo: rotate
    let q = abs(rotated) - scale;
    return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// utils
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h: f32 = clamp(0.5 + 0.5*(a-b)/k, 0.0, 1.0);
    return mix(a, b, h) - k*h*(1.0-h);
}