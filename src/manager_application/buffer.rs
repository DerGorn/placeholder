use crate::graphics_provider::{
    BufferWriter, Index, IndexBufferWriter, Vertex, VertexBufferWriter,
};

#[derive(Debug)]
pub struct IndexBuffer {
    indices: Vec<u8>,
    num_indices: u32,
}
impl IndexBuffer {
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
            num_indices: 0,
        }
    }
    pub fn extend_from_slice<I: Index>(&mut self, new_indices: &[I]) {
        self.num_indices += new_indices.len() as u32;
        self.indices
            .extend_from_slice(bytemuck::cast_slice(new_indices));
    }
    pub fn len(&self) -> u32 {
        self.num_indices
    }
}
impl BufferWriter for IndexBuffer {
    fn buffer_len(&self) -> u32 {
        self.num_indices
    }

    fn buffer_data<'a>(&'a self) -> Option<&'a [u8]> {
        Some(&self.indices)
    }
}
impl IndexBufferWriter for IndexBuffer {}

#[derive(Debug)]
pub struct VertexBuffer {
    vertices: Vec<u8>,
    num_vertices: u32,
}
impl VertexBuffer {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            num_vertices: 0,
        }
    }
    pub fn extend_from_slice<V: Vertex>(&mut self, new_vertices: &[V]) {
        self.num_vertices += new_vertices.len() as u32;
        self.vertices
            .extend_from_slice(bytemuck::cast_slice(new_vertices));
    }
    pub fn len(&self) -> u32 {
        self.num_vertices
    }
}
impl BufferWriter for VertexBuffer {
    fn buffer_len(&self) -> u32 {
        self.num_vertices
    }

    fn buffer_data<'a>(&'a self) -> Option<&'a [u8]> {
        Some(&self.vertices)
    }
}
impl VertexBufferWriter for VertexBuffer {}
