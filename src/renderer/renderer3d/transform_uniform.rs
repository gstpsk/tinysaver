#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

impl TransformUniform {
    pub fn new() -> Self {
        Self {
            model: glam::Mat4::IDENTITY.to_cols_array_2d(),
            view: glam::Mat4::IDENTITY.to_cols_array_2d(),
            projection: glam::Mat4::IDENTITY.to_cols_array_2d(),
            normal_matrix: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}
