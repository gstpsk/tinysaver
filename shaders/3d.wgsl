struct Transform {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> transform: Transform;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) face_normal: vec3<f32>,
    @location(2) in_position: vec3<f32>,
};

@vertex
fn vs_main(@location(0) in_position: vec3<f32>, @location(1) in_normal: vec3<f32>) -> VsOut {
    var out: VsOut;
    let world_pos =
        transform.model * vec4<f32>(in_position, 1.0);

    out.position =
        transform.projection *
        transform.view *
        world_pos;

    out.normal = normalize((transform.normal_matrix * vec4<f32>(in_normal, 0.0)).xyz);

    out.face_normal = in_normal; // pass original normals as-is

    out.in_position = in_position;

    return out;
}

@fragment
fn fs_main2(
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) face_normal: vec3<f32>
) -> @location(0) vec4<f32> {

    return vec4<f32>(
        position.x / 2560.0,
        position.y / 2560.0,
        position.z / 2560.0,
        1.0
    );
}

@fragment
fn fs_main(
    @location(0) normal: vec3<f32>,
    @location(1) face_normal: vec3<f32>
) -> @location(0) vec4<f32> {

    let n = normalize(face_normal);

    if n.z > 0.5 {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0); // front = red
    }

    if n.z < -0.5 {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0); // back = blue
    }

    if n.x < -0.5 {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0); // left = green
    }

    if n.x > 0.5 {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0); // right = yellow
    }

    if n.y > 0.5 {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0); // top = magenta
    }

    return vec4<f32>(0.0, 1.0, 1.0, 1.0); // bottom = cyan
}