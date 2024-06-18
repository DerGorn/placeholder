use crate::create_name_struct;

use super::{IndexBufferWriter, VertexBufferWriter};

create_name_struct!(RenderSceneName);

pub struct RenderScene {
    name: RenderSceneName,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    num_vertices: u32,
    index_format: wgpu::IndexFormat,
    vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
    use_textures: bool,
}
impl RenderScene {
    pub fn new(
        name: RenderSceneName,
        render_pipeline: wgpu::RenderPipeline,
        vertex_buffer: wgpu::Buffer,
        index_buffer: wgpu::Buffer,
        num_indices: u32,
        num_vertices: u32,
        index_format: wgpu::IndexFormat,
        vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
        use_textures: bool,
    ) -> Self {
        Self {
            name,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            num_vertices,
            index_format,
            vertex_buffer_layout,
            use_textures,
        }
    }

    pub fn vertex_buffer_layout(&self) -> &wgpu::VertexBufferLayout {
        &self.vertex_buffer_layout
    }

    pub fn update_pipeline(&mut self, render_pipeline: wgpu::RenderPipeline) {
        self.render_pipeline = render_pipeline;
    }

    pub fn name(&self) -> &RenderSceneName {
        &self.name
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &impl VertexBufferWriter,
        indices: &impl IndexBufferWriter,
    ) {
        if let Some((index_buffer, num_indices)) = indices.write_buffer(
            device,
            queue,
            &self.index_buffer,
            self.num_indices,
            wgpu::BufferUsages::INDEX,
        ) {
            self.index_buffer = index_buffer;
            self.num_indices = num_indices;
        };
        if let Some((vertex_buffer, num_vertices)) = vertices.write_buffer(
            device,
            queue,
            &self.vertex_buffer,
            self.num_vertices,
            wgpu::BufferUsages::VERTEX,
        ) {
            self.vertex_buffer = vertex_buffer;
            self.num_vertices = num_vertices;
        };
    }

    pub fn write_render_pass<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        bind_groups: &[&'a wgpu::BindGroup],
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        for (i, bind_group) in bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, bind_group, &[]);
        }
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), self.index_format);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
