#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub rotation: f32,
    pub color: [f32; 4],
    pub shape_type: u32,
    pub texture_index: u32,
}

impl InstanceData {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {

        const POSITION_OFFSET: wgpu::BufferAddress = 0;
        const SCALE_OFFSET: wgpu::BufferAddress = POSITION_OFFSET+ (size_of::<[f32; 2]>() as wgpu::BufferAddress);
        const ROTATION_OFFSET: wgpu::BufferAddress = SCALE_OFFSET + (size_of::<[f32; 2]>() as wgpu::BufferAddress);
        const COLOR_OFFSET: wgpu::BufferAddress = ROTATION_OFFSET + (size_of::<f32>() as wgpu::BufferAddress);
        const SHAPE_TYPE_OFFSET: wgpu::BufferAddress = COLOR_OFFSET + (size_of::<[f32; 4]>() as wgpu::BufferAddress);
        const TEXTURE_INDEX_OFFSET: wgpu::BufferAddress = SHAPE_TYPE_OFFSET + (size_of::<u32>() as wgpu::BufferAddress);

        const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
            // position
            wgpu::VertexAttribute {
                offset: POSITION_OFFSET,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
            // scale
            wgpu::VertexAttribute {
                offset: SCALE_OFFSET,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x2,
            },
            // rotation
            wgpu::VertexAttribute {
                offset: ROTATION_OFFSET,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32,
            },
            // color
            wgpu::VertexAttribute {
                offset: COLOR_OFFSET,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
            // shape_type
            wgpu::VertexAttribute {
                offset: SHAPE_TYPE_OFFSET,
                shader_location: 6,
                format: wgpu::VertexFormat::Uint32,
            },
            // texture_index
            wgpu::VertexAttribute {
                offset: TEXTURE_INDEX_OFFSET,
                shader_location: 7,
                format: wgpu::VertexFormat::Uint32,
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