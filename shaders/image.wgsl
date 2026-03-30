@group(0) @binding(0)
var textures: binding_array<texture_2d<f32>>;

@group(0) @binding(1)
var image_sampler: sampler;

// used to convert pixel coordinates
// to clip space coordinates
struct Projection {
    matrix: mat4x4<f32>
};

@group(1) @binding(0)
var<uniform> projection: Projection;

// passed by the vertex shader to the rasterizer
// which then passes it on to the fragment shader.
// important to understand is that it is not really
// returned by the vertex shader in the common sense
// pos is used by the rasterizer to place the triangle
// uv is interpolated for each pixel before being passed
// to the fragment shader.
struct VsOut {
    @builtin(position) vertex_position: vec4<f32>,  // clip space position of the vertex
    @location(0) vertex_uv: vec2<f32>,              // interpolated value of where in the texture the pixel data should come from (texture coordinate): (0, 0) is top left
    @location(5) instance_color: vec4<f32>,         // pass the color on to the fragment shader as well
    @location(7) instance_texture_index: u32     
};

@vertex
fn vs_main(
    // quad vertex buffer
    @location(0) in_position: vec2<f32>, 
    @location(1) in_uv: vec2<f32>,
    // instance vertex buffer
    @location(2) instance_position: vec2<f32>,
    @location(3) instance_scale: vec2<f32>,
    @location(4) instance_rotation: f32,
    @location(5) instance_color: vec4<f32>,
    @location(6) instance_shape_type: u32,
    @location(7) instance_texture_index: u32,
) -> VsOut {
    // define return struct
    // prevents wgsl-analyzer from complaining...
    var out: VsOut;

    // apply scaling
    let pos = in_position * instance_scale;

    // apply rotation
    let s = sin(instance_rotation);
    let c = cos(instance_rotation);
    let rotated = vec2<f32>(
        pos.x * c - pos.y * s,
        pos.x * s + pos.y * c
    );

    // apply translation
    let new_position = rotated + instance_position;
    
    // now we use the projection matrix to convert from pixel to clip space
    let clip_space_position = projection.matrix * vec4<f32>(new_position, 0.0, 1.0);

    out.vertex_position = clip_space_position;
    out.vertex_uv = in_uv;
    out.instance_color = instance_color;
    out.instance_texture_index = instance_texture_index;

    return out;
}

// the fragment shader is called thousands of times in parallel
// for each pixel / fragment inside the triangles drawn by the rasterizer
// input: the texture coordinate of the pixel to be drawn
// output: an vertex containing the final rgba data drawn to the screen 
@fragment
fn fs_textured(@location(0) in_uv: vec2<f32>, @location(5) instance_color: vec4<f32>, @location(7) instance_texture_index: u32) -> @location(0) vec4<f32> {
    let fragment = textureSample(textures[instance_texture_index], image_sampler, in_uv);
    let tinted_fragment = fragment * instance_color;
    return tinted_fragment;
}

@fragment
fn fs_solid(@location(5) instance_color: vec4<f32>) -> @location(0) vec4<f32> {
    return instance_color;
}