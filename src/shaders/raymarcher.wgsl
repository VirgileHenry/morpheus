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

// frag shader


struct ScreenResolution {
    width: u32,
    height: u32,
}

@group(1) @binding(0)
var<uniform> screen_resolution: ScreenResolution;

struct CsgObject {
    csg_id: u32,
    data: array<f32, 7>, // hand made union thingy
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

@fragment
fn fs_main(@builtin(position) in: vec4<f32>) -> GBufferOut {
    
    let screen_pos = vec2(in.x / f32(screen_resolution.width) - 0.5, in.y / f32(screen_resolution.height) - 0.5);
    let ray: Ray = get_ray(screen_pos);

    var out: GBufferOut;
    var stack: array<u32, 20>;

    // todo : advance ray towards first box encounter
    // todo : stop condition: out of the box

    let max_iter = 100;
    let hit_eps = 0.00001;
    var eval_point: vec3<f32> = ray.origin;
    for(var i = 0; i < max_iter; i++) {
        let scene_sdf = scene_sdf(eval_point);
        if(scene_sdf < hit_eps) {
            // it's a hit !
            out.albedo = vec4(1.0); // todo : materials
            let depth = length(eval_point - camera.position.xyz);
            out.normal_depth = vec4(scene_normal(eval_point), depth);
            return out;
        }
        eval_point += ray.dir * scene_sdf;
    }

    // infinity, discard
    discard;

}

fn scene_sdf(at: vec3<f32>) -> f32 {
    // we are going to use a stack with reversed polish notations
    var stack_ptr: u32 = 0u;
    var sdf_stack: array<f32, 10>;

    // the csg nodes are written in a prefixed manner,
    // and our stack based approach need a suffix one.
    // so iterate in reverse
    for(var i: u32 = 0u; i < csg_object_count; i++) {

        switch csg_objects[i].csg_id {
            case 0u: { return 1.0 / 0.0; } // 0 is empty object, return inf
            case 1u: { // id 1 is sphere, push it on the stack
                sdf_stack[stack_ptr] = sphere_sdf(at, i);
                stack_ptr += 1u;
            }

            case 4u: { // id 4 is min, from the two values on the stack
                sdf_stack[stack_ptr - 2u] = min(sdf_stack[stack_ptr - 2u], sdf_stack[stack_ptr - 1u]);
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
    let h: f32 = 0.0001; // replace by an appropriate value
    let k: vec2<f32> = vec2(1.0, -1.0);
    return normalize( k.xyy * scene_sdf( at + k.xyy * h ) + 
                      k.yyx * scene_sdf( at + k.yyx * h ) + 
                      k.yxy * scene_sdf( at + k.yxy * h ) + 
                      k.xxx * scene_sdf( at + k.xxx * h ) );
}


// all objects sdf


fn sphere_sdf(at: vec3<f32>, csg_index: u32) -> f32 {
    // we are reading this primitive, so increase the index
    let data: array<f32, 7> = csg_objects[csg_index].data;
    let offset: vec3<f32> = vec3(data[0], data[1], data[2]);
    let radius = data[3];
    return length(offset - at) - radius;
}

fn sphere_normal(at: vec3<f32>, csg_index: u32) -> vec3<f32> {
    let data: array<f32, 7> = csg_objects[csg_index].data;
    let center: vec3<f32> = vec3(data[0], data[1], data[2]);
    return normalize(at - center);
}
