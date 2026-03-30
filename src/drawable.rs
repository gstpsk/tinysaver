use crate::renderer::InstanceData;

#[derive(Copy, Clone)]
pub enum Shape {
    Rectangle { 
        width: f32, 
        height: f32 
    }
}

impl Shape {
    pub fn width(&self) -> f32 {
        match *self {
            Shape::Rectangle { width, .. } => width,
        }
    }

    pub fn height(&self) -> f32 {
        match *self {
            Shape::Rectangle { height, .. } => height,
        }
    }
}

pub enum Material {
    Solid,
    Textured {
        texture_index: u32
    }
}

pub struct Drawable {
    pub shape: Shape,
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub color: (u8, u8, u8),
    pub alpha: u8,
    pub material: Material
}

impl Drawable {
    pub fn new(
        shape: Shape,
        x: f32,
        y: f32,
        color: (u8,u8,u8),
        alpha: u8,
        material: Material
    ) -> Self {
        let (scale_x, scale_y) = match shape {
            Shape::Rectangle { width, height } => (width, height),
        };

        Self {
            shape,
            x,
            y,
            rotation: 0.0,
            scale_x,
            scale_y,
            color,
            alpha,
            material
        }
    }

    // pub fn pipeline_type(&self) -> renderer::PipelineType {
    //     renderer::PipelineType::Solid
    // }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_scale(&mut self, scale_x: f32, scale_y: f32) {
        self.scale_x = scale_x;
        self.scale_y = scale_y;
    }

    pub fn set_color(&mut self, rgb: (u8, u8, u8)) {
        self.color.0 = rgb.0;
        self.color.1 = rgb.1;
        self.color.2 = rgb.2;        
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.alpha = alpha;
    }

    pub fn shape_type(&self) -> u32 {
        match self.shape {
            Shape::Rectangle { .. } => 0,
        }
    }

    pub fn texture_index(&self) -> u32 {
        match self.material {
            Material::Solid => 0,
            Material::Textured { texture_index } => texture_index,
        }
    }

    pub fn to_instance_data(&self) -> InstanceData {
        InstanceData {
            position: [self.x, self.y],
            scale: [self.scale_x, self.scale_y],
            rotation: self.rotation,
            color: [
                self.color.0 as f32 / 255.0,
                self.color.1 as f32 / 255.0,
                self.color.2 as f32 / 255.0,
                self.alpha as f32 / 255.0,
            ],
            shape_type: self.shape_type(),
            texture_index: self.texture_index(),
        }
    }
}