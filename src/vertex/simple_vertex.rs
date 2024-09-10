use ferride_core::{graphics::Vertex, reexports::wgpu::vertex_attr_array};
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
const UI_VERTEX_ATTRIBUTES: [ferride_core::reexports::wgpu::VertexAttribute; 1] =
    vertex_attr_array![0 => Float32x2];
impl Vertex for SimpleVertex {
    fn attributes() -> &'static [ferride_core::reexports::wgpu::VertexAttribute] {
        &UI_VERTEX_ATTRIBUTES
    }
}
