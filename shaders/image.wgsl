@group(0) @binding(0)
var textures: binding_array<texture_2d<f32>>;

@group(0) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(0)
var<uniform> pixel_to_clip: mat4x4<f32>;

struct VsOut {
    @builtin(position) vertex_position: vec4<f32>,      // clip space position of the vertex

    @location(0) quad_uv: vec2<f32>,
    @location(1) instance_color: vec4<f32>,
    @location(2) instance_texture_index: u32,
    @location(3) instance_texture_uv_min: vec2<f32>,
    @location(4) instance_texture_uv_max: vec2<f32>,
};

struct FsIn {
    @builtin(position) fragment_position: vec4<f32>,    // clip space position of the fragment

    @location(0) quad_uv: vec2<f32>,                    // interpolated value of where on the quad we are located
    @location(1) instance_color: vec4<f32>,
    @location(2) instance_texture_index: u32,
    @location(3) instance_texture_uv_min: vec2<f32>,    // fixed value of where on the texture we need to start sampling
    @location(4) instance_texture_uv_max: vec2<f32>,    // fixed value of where on the texture we need to stop sampling
}

@vertex
fn vs_main(
    // quad vertex buffer
    @location(0) local_position: vec2<f32>,             // position of the quad vertices in local space, so (1, 0) for the top right corner
    @location(1) quad_uv: vec2<f32>,                    // position on the quad itself, so (0.5, 0.5) is on the center of the quad.
    // instance vertex buffer
    @location(2) instance_position: vec2<f32>,          // position of the instance's top left corner in screen space
    @location(3) instance_size: vec2<f32>,              // width and height of the instance in pixels
    @location(4) instance_rotation: f32,                // rotation of the quad in radians
    @location(5) instance_color: vec4<f32>,             // RGBA color/tint of the instance, normalized to 0..1
    @location(6) instance_texture_index: u32,           // index of the texture to sample from textures[]
    @location(7) instance_texture_uv_min: vec2<f32>,    // starting position on the texture, where on the texture should the sampler start sampling?
    @location(8) instance_texture_uv_max: vec2<f32>,    // ending position on the texture, where on the texture should the sampler stop sampling?
) -> VsOut {
    var out: VsOut;
    
    // construct matrix to apply rotation
    let rotation_matrix = mat2x2<f32>(
        cos(instance_rotation),  sin(instance_rotation),
       -sin(instance_rotation),  cos(instance_rotation)
    );

    // scale the local vertex, rotate it around the origin,
    // then move it to the instance's screen position
    let screen_position = instance_position + rotation_matrix * (local_position * instance_size);

    // now we use a matrix to convert from screen space to clip space (2560, 1440) -> (1, -1)
    let clip_space_position = pixel_to_clip * vec4<f32>(screen_position, 0.0, 1.0);

    out.vertex_position = clip_space_position;
    out.quad_uv = quad_uv;
    out.instance_color = instance_color;
    out.instance_texture_index = instance_texture_index;
    out.instance_texture_uv_min = instance_texture_uv_min;
    out.instance_texture_uv_max = instance_texture_uv_max;

    return out;
}

@fragment
fn fs_textured(in: FsIn) -> @location(0) vec4<f32> {
    // mix is a function that linearly interpolates between two values
    // basically, given min_uv and max_uv, give me a point in between based on quad_uv, which tells us where in the quad we are
    let texture_uv = mix(
        in.instance_texture_uv_min,
        in.instance_texture_uv_max,
        in.quad_uv
    );

    let fragment = textureSample(
        textures[in.instance_texture_index],
        texture_sampler,
        texture_uv
    );

    return fragment * in.instance_color;
}

@fragment
fn fs_solid(in: FsIn) -> @location(0) vec4<f32> {
    return in.instance_color;
}