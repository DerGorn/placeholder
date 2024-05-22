use wgpu::util::DeviceExt;
use std::fmt::Debug;
use super::{Index, Vertex};

pub trait WindowSurface<I: Index, V: Vertex>: Debug {
    fn surface<'a, 'b: 'a>(&'b self) -> &'a wgpu::Surface<'a>;
    fn size(&self) -> &winit::dpi::PhysicalSize<u32>;
    fn size_mut(&mut self) -> &mut winit::dpi::PhysicalSize<u32>;
    fn config(&self) -> &wgpu::SurfaceConfiguration;
    fn config_mut(&mut self) -> &mut wgpu::SurfaceConfiguration;
    fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>, device: &wgpu::Device) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size_mut().width = new_size.width;
        self.size_mut().height = new_size.height;
        self.config_mut().width = new_size.width;
        self.config_mut().height = new_size.height;
        self.surface().configure(device, self.config());
    }
    fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: Option<&[V]>,
        indices: Option<&[I]>,
    );
    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
}

pub struct Surface<'a, I: Index, V: Vertex> {
    pub wgpu_surface: wgpu::Surface<'a>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
    pub _phantom: std::marker::PhantomData<(I, V)>,
}
impl<I: Index, V: Vertex> Debug for Surface<'_, I, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface")
            .field("size", &self.size)
            .field("config", &self.config)
            .finish()
    }
}
impl<'a, I: Index, V: Vertex> WindowSurface<I, V> for Surface<'a, I, V> {
    fn surface<'b, 'c: 'b>(&'c self) -> &'b wgpu::Surface<'b> {
        &self.wgpu_surface
    }

    fn size_mut(&mut self) -> &mut winit::dpi::PhysicalSize<u32> {
        &mut self.size
    }

    fn size(&self) -> &winit::dpi::PhysicalSize<u32> {
        &self.size
    }

    fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    fn config_mut(&mut self) -> &mut wgpu::SurfaceConfiguration {
        &mut self.config
    }

    fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: Option<&[V]>,
        indices: Option<&[I]>,
    ) {
        if let Some(indices) = indices {
            if self.num_indices < indices.len() as u32 {
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                });
                self.index_buffer = index_buffer;
                self.num_indices = indices.len() as u32;
            } else {
                let indices = bytemuck::cast_slice(indices);
                queue.write_buffer(&self.index_buffer, 0, indices);
            }
        }
        if let Some(vertices) = vertices {
            if self.num_vertices < vertices.len() as u32 {
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
                self.vertex_buffer = vertex_buffer;
                self.num_vertices = vertices.len() as u32;
            } else {
                let vertices = bytemuck::cast_slice(vertices);
                queue.write_buffer(&self.vertex_buffer, 0, vertices);
            }
        }
    }

    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let output = self.surface().get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), I::index_format());
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
