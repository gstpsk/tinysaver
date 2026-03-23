@group(0) @binding(0)
var image_texture: texture_2d<f32>;

@group(0) @binding(1)
var image_sampler: sampler;

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

@vertex
fn vs_main(@location(0) in_position: vec2<f32>, @location(1) in_uv: vec2<f32>) -> VsOut {
    // define return struct
    // prevents wgsl-analyzer from complaining...
    var out: VsOut;
    
    let clip_space_position = vec4<f32>(in_position, 0.0, 1.0); // convert 2D coordinate to clip space

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
    return textureSample(image_texture, image_sampler, in_uv);
}