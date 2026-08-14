// we use repr(C) to prevent Rust from messing with the memory layout
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex3D {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex3D>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // @location(0) position: vec3<f32>
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // @location(1) normal: vec3<f32>
                wgpu::VertexAttribute {
                    offset: 12, // Previous attribute has 3 floats of 4 bytes is 3x4 = 12 bytes (f32 = 32 bits = 4 bytes) 
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // @location(2) uv: vec2<f32>
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }
}