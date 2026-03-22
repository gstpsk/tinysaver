// take the framebuffer of pixels as input
@group(0) @binding(0)
var input_tex: texture_2d<f32>;

// sampler tells the GPU how to read the texture?
@group(0) @binding(1)
var input_sampler: sampler;

// pass vertex output to fragment input
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle vertex shader
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VsOut {
    // Convert from [-1,1] clip space to [0,1] UV space
    var uv = (position + vec2(1.0, 1.0)) * 0.5;
    // flip the output
    uv.y = 1.0 - uv.y;

    return VsOut(
        vec4<f32>(position, 0.0, 1.0),
        uv
    );
}

// color the frame buffer white
@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {

    // sample the color
    // i think basically we get the color for each pixel simultaneously
    let color = textureSample(input_tex, input_sampler, in.uv);

   //return color;

    if (color.a > 0.0) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else {
        return color;
    }
}
