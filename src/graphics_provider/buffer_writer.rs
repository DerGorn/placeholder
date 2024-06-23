use wgpu::util::DeviceExt;

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
        force_overwrite: bool,
    ) -> Option<(wgpu::Buffer, u32)> {
        if let Some(buffer_data) = self.buffer_data() {
            let new_len = self.buffer_len();
            if buffer_len < new_len || force_overwrite {
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

pub trait VertexBufferWriter: BufferWriter {}
