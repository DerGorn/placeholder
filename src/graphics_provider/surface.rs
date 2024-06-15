use crate::create_name_struct;

use super::ShaderDescriptor;
use std::fmt::Debug;
use wgpu::util::DeviceExt;

pub trait WindowSurface: Debug {
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
    fn create_render_pipeline<'a>(
        &self,
        device: &wgpu::Device,
        bind_group_layout: &[&wgpu::BindGroupLayout],
        shader: &wgpu::ShaderModule,
        shader_descriptor: &ShaderDescriptor,
        vertex_buffer_layout: wgpu::VertexBufferLayout<'a>,
    ) -> wgpu::RenderPipeline;
    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_scenes: &[&RenderScene],
        bind_groups: &[&wgpu::BindGroup],
    );
}

pub struct Surface<'a> {
    pub wgpu_surface: wgpu::Surface<'a>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub config: wgpu::SurfaceConfiguration,
}
impl Debug for Surface<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface")
            .field("size", &self.size)
            .field("config", &self.config)
            .finish()
    }
}
impl<'a> WindowSurface for Surface<'a> {
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

    fn create_render_pipeline<'b>(
        &self,
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        shader: &wgpu::ShaderModule,
        shader_descriptor: &ShaderDescriptor,
        vertex_buffer_layout: wgpu::VertexBufferLayout<'b>,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: shader_descriptor.vertex_shader,
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: shader_descriptor.fragment_shader,
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        render_pipeline
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_scenes: &[&RenderScene],
        bind_groups: &[&wgpu::BindGroup],
    ) {
        let output = self
            .surface()
            .get_current_texture()
            .expect("Our food has no texture");
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

            for render_scene in render_scenes {
                render_scene.write_render_pass(&mut render_pass, bind_groups);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

create_name_struct!(RenderSceneName);

pub struct RenderScene {
    name: RenderSceneName,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    num_vertices: u32,
    index_format: wgpu::IndexFormat,
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
    ) -> Self {
        Self {
            name,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            num_vertices,
            index_format,
        }
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
        vertices: &dyn BufferWriter,
        indices: &dyn BufferWriter,
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

    fn write_render_pass<'a>(
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
