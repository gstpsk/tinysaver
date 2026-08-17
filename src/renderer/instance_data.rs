#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub color: [f32; 4],
    pub texture_index: u32,
    pub instance_texture_uv_min: [f32; 2],
    pub instance_texture_uv_max: [f32; 2],
}

impl InstanceData {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {

        const POSITION_OFFSET: wgpu::BufferAddress          = 0;
        const SIZE_OFFSET: wgpu::BufferAddress              = POSITION_OFFSET           + (size_of::<[f32; 2]>() as wgpu::BufferAddress);
        const ROTATION_OFFSET: wgpu::BufferAddress          = SIZE_OFFSET               + (size_of::<[f32; 2]>() as wgpu::BufferAddress);
        const COLOR_OFFSET: wgpu::BufferAddress             = ROTATION_OFFSET           + (size_of::<f32>() as wgpu::BufferAddress);
        const TEXTURE_INDEX_OFFSET: wgpu::BufferAddress     = COLOR_OFFSET              + (size_of::<[f32; 4]>() as wgpu::BufferAddress);
        const INSTANCE_MIN_UV_OFFSET: wgpu::BufferAddress   = TEXTURE_INDEX_OFFSET      + (size_of::<u32>() as wgpu::BufferAddress);
        const INSTANCE_MAX_UV_OFFSET: wgpu::BufferAddress   = INSTANCE_MIN_UV_OFFSET    + (size_of::<[f32; 2]>() as wgpu::BufferAddress);

        const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
            // position
            wgpu::VertexAttribute {
                offset: POSITION_OFFSET,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
            // instance_size
            wgpu::VertexAttribute {
                offset: SIZE_OFFSET,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x2,
            },
            // instance_rotation
            wgpu::VertexAttribute {
                offset: ROTATION_OFFSET,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32,
            },
            // instance_color
            wgpu::VertexAttribute {
                offset: COLOR_OFFSET,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
            // instance_texture_index
            wgpu::VertexAttribute {
                offset: TEXTURE_INDEX_OFFSET,
                shader_location: 6,
                format: wgpu::VertexFormat::Uint32,
            },
            // instance_texture_min_uv
            wgpu::VertexAttribute {
                offset: INSTANCE_MIN_UV_OFFSET,
                shader_location: 7,
                format: wgpu::VertexFormat::Float32x2,
            },
            // instance_texture_max_uv
            wgpu::VertexAttribute {
                offset: INSTANCE_MAX_UV_OFFSET,
                shader_location: 8,
                format: wgpu::VertexFormat::Float32x2,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceData>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ATTRIBUTES,
        }
    }
}

pub struct InstanceBatch {
    pub solid: Vec<InstanceData>,
    pub textured: Vec<InstanceData>,
    pub wireframe: Vec<InstanceData>
}