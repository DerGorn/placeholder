#![allow(deprecated)]
use std::fs;
use wgpu::util::DeviceExt;
use wgpu::rwh::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::{Window, WindowId};

mod buffer_primitives;
pub use buffer_primitives::{Index, Vertex};

mod surface;
use surface::{Surface, WindowSurface};

mod shader_descriptor;
pub use shader_descriptor::ShaderDescriptor;

pub struct GraphicsProvider<I: Index, V: Vertex> {
    instance: wgpu::Instance,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surfaces: Vec<(WindowId, Box<dyn WindowSurface<I, V>>)>,
}
impl<I: Index, V: Vertex> GraphicsProvider<I, V> {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        Self {
            instance,
            adapter: None,
            device: None,
            queue: None,
            surfaces: Vec::new(),
        }
    }

    fn init(&mut self, surface: &wgpu::Surface) {
        let adapter = futures::executor::block_on(self.instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ))
        .unwrap();

        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        ))
        .unwrap();
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
    }

    pub fn init_window(&mut self, window: &Window, shader_descriptor: &ShaderDescriptor) {
        let size = window.inner_size();
        //#Safety
        //
        //Should be safe if surface discarded when window is destroyed
        let surface = unsafe {
            self.instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: window.raw_display_handle().unwrap(),
                    raw_window_handle: window.raw_window_handle().unwrap(),
                })
        }
        .unwrap();

        if self.adapter.is_none() {
            self.init(&surface);
        }

        let capabilities = surface.get_capabilities(&self.adapter.as_ref().unwrap());
        let format = capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader =
            self.device
                .as_ref()
                .unwrap()
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(&format!("Shader Module {:?}", shader_descriptor.file)),
                    source: wgpu::ShaderSource::Wgsl(
                        fs::read_to_string(shader_descriptor.file)
                            .expect(&format!("Could not load '{}'\n", shader_descriptor.file))
                            .into(),
                    ),
                });
        let pipeline_layout =
            self.device
                .as_ref()
                .unwrap()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });
        let render_pipeline =
            self.device
                .as_ref()
                .unwrap()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(&format!("Render Pipeline {:?}", window.id())),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: shader_descriptor.vertex_shader,
                        buffers: &[V::describe_buffer_layout()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: shader_descriptor.fragment_shader,
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
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

        let vertex_buffer =
            self.device
                .as_ref()
                .unwrap()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Vertex Buffer {:?}", window.id())),
                    contents: &[],
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
        let index_buffer =
            self.device
                .as_ref()
                .unwrap()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Index Buffer {:?}", window.id())),
                    contents: &[],
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                });
        let num_vertices = 0;
        let num_indices = 0;

        self.surfaces.push((
            window.id(),
            Box::new(Surface {
                wgpu_surface: surface,
                size,
                config,
                render_pipeline,
                vertex_buffer,
                index_buffer,
                num_vertices,
                num_indices,
                _phantom: std::marker::PhantomData,
            }),
        ));
    }

    pub fn resize_window(&mut self, id: &WindowId, new_size: &winit::dpi::PhysicalSize<u32>) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let Some(device) = &self.device {
                surface.resize(new_size, device);
            }
        }
    }

    pub fn render_window(&mut self, id: &WindowId) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
                surface.render(device, queue);
            }
        }
    }

    pub fn update_buffers(&mut self, id: &WindowId, vertices: Option<&[V]>, indices: Option<&[I]>) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
                surface.update(device, queue, vertices, indices)
            }
        }
    }

    pub fn remove_window(&mut self, id: &WindowId) {
        self.surfaces.retain(|(i, _)| i != id);
    }
}


