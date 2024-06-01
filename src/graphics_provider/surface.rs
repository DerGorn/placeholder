use super::{texture::TextureProvider, Index, ShaderDescriptor, Vertex};
use std::fmt::Debug;
use wgpu::util::DeviceExt;

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
    fn create_render_pipeline(
        &mut self,
        device: &wgpu::Device,
        bind_group_layout: &[&wgpu::BindGroupLayout],
    );
    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_groups: &[&wgpu::BindGroup],
    );
}

pub struct Surface<'a, I: Index, V: Vertex> {
    pub wgpu_surface: wgpu::Surface<'a>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub config: wgpu::SurfaceConfiguration,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
    pub shader: wgpu::ShaderModule,
    pub shader_descriptor: ShaderDescriptor,
    pub render_pipeline: Option<wgpu::RenderPipeline>,
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

    fn create_render_pipeline(
        &mut self,
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.shader,
                entry_point: self.shader_descriptor.vertex_shader,
                buffers: &[V::describe_buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.shader,
                entry_point: self.shader_descriptor.fragment_shader,
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

        self.render_pipeline = Some(render_pipeline);
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
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

            if let Some(render_pipeline) = self.render_pipeline.as_ref() {
                render_pass.set_pipeline(render_pipeline);
            }
            for (i, bind_group) in bind_groups.iter().enumerate() {
                println!("Bind group: {} {:?}", i, bind_group);
                render_pass.set_bind_group(i as u32, bind_group, &[]);
            }
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), I::index_format());
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
