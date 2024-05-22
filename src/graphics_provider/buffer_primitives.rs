use std::fmt::Debug;

pub trait Vertex:
    Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable + repr_trait::C
{
    fn describe_buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}
pub trait Index: Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable {
    fn index_format() -> wgpu::IndexFormat;
}
impl Index for u16 {
    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }
}
impl Index for u32 {
    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint32
    }
}
