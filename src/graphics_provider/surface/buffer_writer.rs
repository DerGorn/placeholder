use wgpu::util::DeviceExt;

use crate::graphics_provider::{Index, Vertex};

pub trait BufferWriter {
    fn buffer_data<'a>(&'a self) -> Option<&'a [u8]>;
    fn buffer_len(&self) -> u32;

    fn write_buffer(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer: &wgpu::Buffer,
        buffer_len: u32,
        usage: wgpu::BufferUsages,
    ) -> Option<(wgpu::Buffer, u32)> {
        if let Some(buffer_data) = self.buffer_data() {
            let new_len = self.buffer_len();
            if buffer_len < new_len {
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Buffer", usage)),
                    contents: buffer_data,
                    usage: usage | wgpu::BufferUsages::COPY_DST,
                });
                Some((buffer, new_len))
            } else {
                queue.write_buffer(buffer, 0, buffer_data);
                None
            }
        } else {
            None
        }
    }
}
impl<T> BufferWriter for Option<&[T]>
where
    T: bytemuck::Pod,
{
    fn buffer_len(&self) -> u32 {
        self.unwrap_or_else(|| &[]).len() as u32
    }
    fn buffer_data<'a>(&'a self) -> Option<&'a [u8]> {
        self.map(|s| bytemuck::cast_slice(s))
    }
}

pub trait IndexBufferWriter: BufferWriter {}
impl<I: Index> IndexBufferWriter for Option<&[I]> {}

pub trait VertexBufferWriter: BufferWriter {}
impl<V: Vertex> VertexBufferWriter for Option<&[V]> {}
