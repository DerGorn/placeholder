use placeholder::graphics::Vertex;
use repr_trait::C;
use threed::Vector;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct SimpleVertex {
    position: [f32; 2],
}
impl SimpleVertex {
    pub fn new(position: Vector<f32>) -> Self {
        Self {
            position: [position.x, position.y],
        }
    }
}
impl Vertex for SimpleVertex {
    fn describe_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                format: wgpu::VertexFormat::Float32x2,
                shader_location: 0,
            }],
        }
    }
}
