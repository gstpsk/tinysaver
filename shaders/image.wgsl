@group(0) @binding(0)
var image_texture: texture_2d<f32>;

@group(0) @binding(1)
var image_sampler: sampler;

@group(0) @binding(2)
var<uniform> transform: Transform;

@group(0) @binding(3)
var<uniform> projection: Projection;

@group(0) @binding(4)
var<uniform> tint: Tint;

// passed by the vertex shader to the rasterizer
// which then passes it on to the fragment shader.
// important to understand is that it is not really
// returned by the vertex shader in the common sense
// pos is used by the rasterizer to place the triangle
// uv is interpolated for each pixel before being passed
// to the fragment shader.
struct VsOut {
    @builtin(position) pos: vec4<f32>,  // clip space position of the vertex
    @location(0) uv: vec2<f32>          // interpolated value of where in the texture the pixel data should come from (texture coordinate): (0, 0) is top left
};

// contains transform data
// could later add scale
// and rotation
struct Transform {
    offset: vec2<f32>
};

// used to convert pixel coordinates
// to clip space coordinates
struct Projection {
    matrix: mat4x4<f32>
};

struct Tint {
    color: vec4<f32>
};

@vertex
fn vs_main(@location(0) in_position: vec2<f32>, @location(1) in_uv: vec2<f32>) -> VsOut {
    // define return struct
    // prevents wgsl-analyzer from complaining...
    var out: VsOut;

    // apply translation, still in pixels
    let new_position = in_position + transform.offset;
    
    // now we use the projection matrix to convert from pixel to clip space
    let clip_space_position = projection.matrix * vec4<f32>(new_position, 0.0, 1.0);

    out.pos = clip_space_position;
    out.uv = in_uv;

    return out;
}

// the fragment shader is called thousands of times in parallel
// for each pixel / fragment inside the triangles drawn by the rasterizer
// input: the texture coordinate of the pixel to be drawn
// output: an vertex containing the final rgba data drawn to the screen 
@fragment
fn fs_main(@location(0) in_uv: vec2<f32>) -> @location(0) vec4<f32> {
    let fragment = textureSample(image_texture, image_sampler, in_uv);
    let tinted_fragment = fragment * tint.color;
    return tinted_fragment;
}

@fragment
fn fs_solid(@location(0) in_uv: vec2<f32>) -> @location(0) vec4<f32> {
    return tint.color;
}