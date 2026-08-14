use crate::renderer::vertex3d::Vertex3D;

pub struct Mesh {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u16>,
}


impl Mesh {
    pub fn cube() -> Self {
        let vertices = vec![
            // front face
            Vertex3D { position: [-0.5,  0.5,  0.5], normal: [0.0, 0.0, 1.0] }, // top left
            Vertex3D { position: [ 0.5,  0.5,  0.5], normal: [0.0, 0.0, 1.0] }, // top right
            Vertex3D { position: [-0.5, -0.5,  0.5], normal: [0.0, 0.0, 1.0] }, // bottom left
            Vertex3D { position: [ 0.5, -0.5,  0.5], normal: [0.0, 0.0, 1.0] }, // bottom right
            // back face
            Vertex3D { position: [-0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0] }, // top left
            Vertex3D { position: [ 0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0] }, // top right
            Vertex3D { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] }, // bottom left
            Vertex3D { position: [ 0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] }, // bottom right

            // left face
            Vertex3D { position: [-0.5,  0.5, -0.5], normal: [-1.0, 0.0, 0.0] },
            Vertex3D { position: [-0.5,  0.5,  0.5], normal: [-1.0, 0.0, 0.0] },
            Vertex3D { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0] },
            Vertex3D { position: [-0.5, -0.5,  0.5], normal: [-1.0, 0.0, 0.0] },

            // right face
            Vertex3D { position: [ 0.5,  0.5,  0.5], normal: [1.0, 0.0, 0.0] },
            Vertex3D { position: [ 0.5,  0.5, -0.5], normal: [1.0, 0.0, 0.0] },
            Vertex3D { position: [ 0.5, -0.5,  0.5], normal: [1.0, 0.0, 0.0] },
            Vertex3D { position: [ 0.5, -0.5, -0.5], normal: [1.0, 0.0, 0.0] },

            // top face
            Vertex3D { position: [-0.5,  0.5, -0.5], normal: [0.0, 1.0, 0.0] },
            Vertex3D { position: [ 0.5,  0.5, -0.5], normal: [0.0, 1.0, 0.0] },
            Vertex3D { position: [-0.5,  0.5,  0.5], normal: [0.0, 1.0, 0.0] },
            Vertex3D { position: [ 0.5,  0.5,  0.5], normal: [0.0, 1.0, 0.0] },

            // bottom face
            Vertex3D { position: [-0.5, -0.5,  0.5], normal: [0.0, -1.0, 0.0] },
            Vertex3D { position: [ 0.5, -0.5,  0.5], normal: [0.0, -1.0, 0.0] },
            Vertex3D { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0] },
            Vertex3D { position: [ 0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0] },
        ];

        let indices: Vec<u16> = vec![
            0, 1, 2, 2, 1, 3,       // front
            4, 6, 5, 5, 6, 7,       // back
            8, 9, 10, 10, 9, 11,    // left
            12, 14, 13, 13, 14, 15, // right
            16, 17, 18, 18, 17, 19, // top
            20, 22, 21, 21, 22, 23, // bottom
        ];

        Self { vertices, indices }
    }

    pub fn wireframe_indices(mesh: &Mesh) -> Vec<u16> {
        let mut line_indices = Vec::new();

        for triangle in mesh.indices.chunks_exact(3) {
            let a = triangle[0];
            let b = triangle[1];
            let c = triangle[2];

            line_indices.push(a);
            line_indices.push(b);

            line_indices.push(b);
            line_indices.push(c);

            line_indices.push(c);
            line_indices.push(a);
        }

        line_indices
    }
}

pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl GpuMesh {
    pub fn from_mesh(device: &wgpu::Device, mesh: &Mesh) -> Self {
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh vertex buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh index buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        }
    }
}