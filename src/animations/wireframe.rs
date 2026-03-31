use font_kit::loaders::default;
use wgpu::Color;
use winit::keyboard::Key;

use crate::{
    animation::Animation,
    drawable::{self, Drawable, Material, Shape},
    renderer::{InstanceBatch, Renderer2D},
};

struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

impl Point3D {
    // increases z value of the point by dz
    fn translate_z(&self, dz: f32) -> Point3D {
        Point3D { x: self.x, y: self.y, z: self.z + dz }
    }

    // projects 3d point on to 2D plane
    fn project(&self, aspect: f32) -> (f32, f32) {
        let x = (self.x) * aspect;
        let y = (self.y);
        (x, y)
    }

    // rotates the point around its x axis
    fn rotate_x(&self, angle: f32) -> Point3D {
        let sin = angle.sin();
        let cos = angle.cos();

        Point3D {
            x: self.x,
            y: self.y * cos - self.z * sin,
            z: self.y * sin + self.z * cos,
        }
    }

    // rotates the point around its y axis
    fn rotate_y(&self, angle: f32) -> Point3D {
        let sin = angle.sin();
        let cos = angle.cos();

        Point3D {
            x: self.x * cos - self.z * sin,
            y: self.y,
            z: self.x * sin + self.z * cos,
        }
    }

    // rotates the point around its z axis
    fn rotate_z(&self, angle: f32) -> Point3D {
        let sin = angle.sin();
        let cos = angle.cos();

        Point3D {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
            z: self.z,
        }
    }
}

struct Cube {
    vertices: Vec<Point3D>,
    edges: Vec<(i32, i32)>,
}

impl Cube {
    pub fn new() -> Self {
        let vertices = vec![
            // front face
            Point3D {x: -0.25, y:  0.25, z:  -0.25}, // top left
            Point3D {x:  0.25, y:  0.25, z:  -0.25}, // top right
            Point3D {x: -0.25, y: -0.25, z:  -0.25}, // bottom left
            Point3D {x:  0.25, y: -0.25, z:  -0.25}, // bottom right
            // back face
            Point3D {x: -0.25, y:  0.25, z:  0.25},
            Point3D {x:  0.25, y:  0.25, z:  0.25},
            Point3D {x: -0.25, y: -0.25, z:  0.25},
            Point3D {x:  0.25, y: -0.25, z:  0.25},
        ];

        let edges = vec![
            (0, 1), (1, 3), (3, 2), (2, 0), // front face
            (4, 5), (5, 7), (7, 6), (6, 4), // back face
            (0, 4), (1, 5), (2, 6), (3, 7), // connections
        ];

        Self {
            vertices,
            edges
        }
    }
}

struct Thing {
    vertices: Vec<Point3D>,
    edges: Vec<(i32, i32)>,
}

impl Thing {
    pub fn new() -> Self {
        let vertices = vec![
            Point3D {x: 0.0, y:  0.25, z:  0.0}, // top middle middle
            Point3D {x: 0.0, y:  -0.25, z:  0.0}, // bottom middle middle
            Point3D {x: -0.25, y: -0.25, z:  -0.25}, // bottom left front
            Point3D {x:  0.25, y: -0.25, z:  -0.25}, // bottom right front
            Point3D {x: -0.25, y: -0.25, z:  0.25}, // bottom left back
            Point3D {x:  0.25, y: -0.25, z:  0.25}, // bottom left back
        ];

        let edges = vec![
            (0, 1),                          // connect top to bottom
            (0, 2), (0, 3), (0, 4), (0, 5), // connect top to corners
            (1, 2), (1, 3), (1, 4), (1, 5), // connect bottom to corners
        ];

        Self {
            vertices,
            edges
        }
    }
}

struct Pyramid {
    vertices: Vec<Point3D>,
    edges: Vec<(i32, i32)>,
}

impl Pyramid {
    pub fn new() -> Self {
        let vertices = vec![
            Point3D {x: 0.0, y:  0.25, z:  0.0}, // top middle middle
            Point3D {x: -0.25, y: -0.25, z:  -0.25}, // bottom left front
            Point3D {x:  0.25, y: -0.25, z:  -0.25}, // bottom right front
            Point3D {x: 0.0, y: -0.25, z:  0.25}, // bottom middle back
        ];

        let vertices = vec![
            Point3D { x: 0.0,    y:  0.25, z:  0.0 },   // top

            Point3D { x: -0.25,  y: -0.25, z: -0.1443 },
            Point3D { x:  0.25,  y: -0.25, z: -0.1443 },
            Point3D { x:  0.0,   y: -0.25, z:  0.2887 },
        ];

        let edges = vec![
            (0, 1), (0, 2), (0, 3),         // connect top to bottom
            (1, 2), (2, 3), (3, 1)          // connect bottom corners
        ];

        Self {
            vertices,
            edges
        }
    }
}

pub struct WireframeAnimation {
    renderer: Renderer2D,
    cube: Cube,
    point_drawables: Vec<Drawable>,
    line_drawables: Vec<Drawable>,
    surface_width: u32,
    surface_height: u32,
    angle_x: f32,
    angle_y: f32,
    angle_z: f32,
    dz: f32,
}

impl WireframeAnimation {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        surface_width: u32,
        surface_height: u32,
    ) -> Self {
        let renderer =
            Renderer2D::new(device, queue, surface_format, surface_width, surface_height);

        let color = (255, 255, 255);
        let alpha = 255;

        let cube = Cube::new();

        let mut point_drawables = Vec::new();
        for _ in &cube.vertices {
            point_drawables.push(Drawable::new(Shape::Rectangle { width: 10.0, height: 10.0 }, 0.0, 0.0, color, alpha, Material::Solid));
        }

        let mut line_drawables = Vec::new();
        for i in 0..cube.edges.len() {
            if i == 4 || i == 5 || i == 6 || i == 7 {
                line_drawables.push(Drawable::new(Shape::Line { dx: 0.0, dy: 0.0, thickness: 3.0 },0.0,0.0,(255, 0, 0),50,Material::Solid));
            } else {
                line_drawables.push(Drawable::new(Shape::Line { dx: 0.0, dy: 0.0, thickness: 3.0 },0.0,0.0,color,128,Material::Solid));
            }
        }

        Self {
            renderer,
            cube,
            point_drawables,
            line_drawables,
            surface_width,
            surface_height,
            angle_x: 0.0,
            angle_y: 0.0,
            angle_z: 0.0,
            dz: 4.0,
        }
    }

    pub fn render(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mut instance_batch = InstanceBatch {
            solid: Vec::new(),
            textured: Vec::new(),
            wireframe: Vec::new(),
        };

        // for d in &self.point_drawables {
        //     instance_batch.solid.push(d.to_instance_data());
        // }

        for d in &self.line_drawables {
            instance_batch.solid.push(d.to_instance_data());
        }

        self.renderer.upload_batches(queue, &instance_batch);

        self.renderer.render(encoder, target, &instance_batch);
    }



    fn to_screen(px: f32, py: f32, surface_width: u32, surface_height: u32) -> (f32, f32) {
        // convert to screen space
        let screen_x = (px + 1.0)/2.0 * surface_width as f32;
        let screen_y = (1.0 - (py + 1.0)/2.0) * surface_height as f32;
        
        (screen_x, screen_y)
    }

    fn update(&mut self) {
        self.angle_y += 0.01;
        
        //self.dz += 0.01;

        let aspect_ratio = self.surface_height as f32 / self.surface_width as f32;

        // create screen positions array with length of amount of vertices in cube
        // these live in screenspace, so width x height
        let mut screen_positions: Vec<(f32, f32, f32)> = vec![(0.0, 0.0, 0.0); self.cube.vertices.len()];

        // transform the cubes vertices
        for i in 0..self.cube.vertices.len() {
            let v = &self.cube.vertices[i];

            let new_v = v.rotate_x(self.angle_x).rotate_y(self.angle_y).rotate_z(self.angle_z).translate_z(self.dz);

            // project 3d point to 2d based on z value
            // respecting the aspect ratio of the surface/canvas/display
            // these are still in ??? space
            let (px, py) = new_v.project(aspect_ratio);

            // now convert to screen space
            let (sx, sy) = Self::to_screen(px, py, self.surface_width, self.surface_height);

            // store this position (to be used by the edges later)
            screen_positions[i] = (sx, sy, new_v.z);

            // and update the drawables position
            self.point_drawables[i].set_position(sx, sy);
        }

        // transform the cubes edges
        for i in 0..self.cube.edges.len() {
            let (start_index, end_index) = self.cube.edges[i];

            let (x1, y1, z1) = screen_positions[start_index as usize];
            let (x2, y2, z2) = screen_positions[end_index as usize];

            // calculate the distance between the points
            let dx = x2 - x1;
            let dy = y2 - y1;

            // update the drawables position
            self.line_drawables[i].set_position(x1, y1);
            // update its length
            self.line_drawables[i].shape = Shape::Line { dx, dy, thickness: 10.0 / ((z1+z2) * 0.5) };
            // and set its rotation
            self.line_drawables[i].rotation = dy.atan2(dx);
        }
    }
}

impl Animation for WireframeAnimation {
    fn update(&mut self, queue: &wgpu::Queue) {
        self.update();
    }

    fn render(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        self.render(queue, encoder, target);
    }

    fn on_key(&mut self, key: Key) {
        match key {
            Key::Character(ref s) if s == "w" => { 
                self.angle_x += 0.02;
                self.angle_y += 0.02;
            },
            Key::Character(ref s) if s == "s" => { 
                self.angle_x -= 0.02;
                self.angle_y -= 0.02;
            }
                ,
            _ => {}
        }        
    }
}
