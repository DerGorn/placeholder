use placeholder::graphics::Vertex as Vert;
use repr_trait::C;
use threed::Vector;

pub struct TextureCoordinates {
    pub u: f32,
    pub v: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    texture: u32,
}
impl Vertex {
    pub fn new(
        position: Vector<f32>,
        texture_coordinates: &TextureCoordinates,
        texture: u32,
    ) -> Self {
        Self {
            position: [position.x, position.y],
            tex_coords: [texture_coordinates.u, texture_coordinates.v],
            texture,
        }
    }
}
impl Vert for Vertex {
    fn describe_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Uint32,
                    shader_location: 2,
                },
            ],
        }
    }
}
