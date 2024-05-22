#![allow(deprecated)]
use std::fmt::Debug;
use std::fs;
use wgpu::rwh::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

mod window_descriptor;
pub use window_descriptor::WindowDescriptor;

mod event_manager;
pub use event_manager::EventManager;

mod window_manager;
pub use window_manager::WindowManager;

trait WindowSurface<I: Index, V: Vertex>: Debug {
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
struct Surface<'a, I: Index, V: Vertex> {
    wgpu_surface: wgpu::Surface<'a>,
    size: winit::dpi::PhysicalSize<u32>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,
    num_indices: u32,
    _phantom: std::marker::PhantomData<(I, V)>,
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
#[derive(Debug, Clone)]
pub struct ShaderDescriptor {
    pub file: &'static str,
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
}
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
pub struct GraphicsProvider<I: Index, V: Vertex> {
    instance: wgpu::Instance,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surfaces: Vec<(WindowId, Box<dyn WindowSurface<I, V>>)>,
}
impl<I: Index, V: Vertex> GraphicsProvider<I, V> {
    fn new() -> Self {
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

    fn init_window(&mut self, window: &Window, shader_descriptor: &ShaderDescriptor) {
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

    fn resize_window(&mut self, id: &WindowId, new_size: &winit::dpi::PhysicalSize<u32>) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let Some(device) = &self.device {
                surface.resize(new_size, device);
            }
        }
    }

    fn render_window(&mut self, id: &WindowId) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
                surface.render(device, queue);
            }
        }
    }

    fn update_buffers(&mut self, id: &WindowId, vertices: Option<&[V]>, indices: Option<&[I]>) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
                surface.update(device, queue, vertices, indices)
            }
        }
    }

    fn remove_window(&mut self, id: &WindowId) {
        self.surfaces.retain(|(i, _)| i != id);
    }
}

pub struct ManagerApplication<
    E: ApplicationEvent<I, V> + 'static,
    M: EventManager<E>,
    I: Index,
    V: Vertex,
> {
    event_manager: M,
    window_manager: WindowManager<E>,
    graphics_provider: GraphicsProvider<I, V>,
}

impl<'a, E: ApplicationEvent<I, V> + 'static, M: EventManager<E>, I: Index, V: Vertex>
    ApplicationHandler<E> for ManagerApplication<E, M, I, V>
{
    fn resumed(&mut self, _active_loop: &ActiveEventLoop) {
        self.window_manager.send_event(E::app_resumed()).unwrap();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self
            .event_manager
            .window_event(&mut self.window_manager, event_loop, &id, &event)
        {
            match event {
                WindowEvent::CloseRequested => {
                    if self.window_manager.amount_windows() == 1 {
                        event_loop.exit();
                    } else {
                        self.graphics_provider.remove_window(&id);
                        self.window_manager.remove_window(&id);
                    }
                }
                WindowEvent::Resized(size) => self.graphics_provider.resize_window(&id, &size),
                WindowEvent::ScaleFactorChanged { .. } => {
                    //TODO: I think the window will be resized  on its own, which fires a Resized event
                }
                WindowEvent::RedrawRequested => {
                    self.graphics_provider.render_window(&id);
                    self.window_manager
                        .get_window(&id)
                        .unwrap()
                        .request_redraw();
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::F11),
                            ..
                        },
                    ..
                } => {
                    let window = self.window_manager.get_window(&id).unwrap();
                    match window.fullscreen() {
                        Some(Fullscreen::Borderless(_)) => {
                            window.set_fullscreen(None);
                        }
                        _ => {
                            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: E) {
        match event.is_request_new_window() {
            Some((window_descriptor, shader_descriptor, name)) => {
                self.create_window(window_descriptor, shader_descriptor, event_loop, name)
            }
            None => {}
        };
        match event.is_render_update() {
            Some((_, None, None)) | None => {}
            Some((id, indices, vertices)) => {
                self.graphics_provider.update_buffers(id, vertices, indices)
            }
        }
        self.event_manager
            .user_event(&mut self.window_manager, event_loop, &event);
    }
}

impl<'a, E: ApplicationEvent<I, V> + 'static, M: EventManager<E>, I: Index, V: Vertex>
    ManagerApplication<E, M, I, V>
{
    pub fn new(event_manager: M) -> Self {
        Self {
            event_manager,
            window_manager: Default::default(),
            graphics_provider: GraphicsProvider::new(),
        }
    }

    pub fn create_window(
        &mut self,
        descriptor: &WindowDescriptor,
        shader_descriptor: &ShaderDescriptor,
        active_loop: &ActiveEventLoop,
        name: &str,
    ) {
        let window = active_loop
            .create_window(descriptor.get_attributes(active_loop))
            .unwrap();
        self.window_manager
            .send_event(E::new_window(&window.id(), name))
            .unwrap();
        self.graphics_provider
            .init_window(&window, shader_descriptor);
        // window.request_redraw();
        self.window_manager.add_window(window);
    }

    pub fn run(&mut self) {
        env_logger::init();
        let event_loop = EventLoop::<E>::with_user_event().build().unwrap();
        let event_loop_proxy = event_loop.create_proxy();
        self.window_manager.set_event_loop(event_loop_proxy);

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(self).unwrap();
    }
}

pub trait ApplicationEvent<I: Index, V: Vertex>: Debug {
    fn app_resumed() -> Self;
    fn new_window(id: &WindowId, name: &str) -> Self;
    fn is_request_new_window<'a>(
        &'a self,
    ) -> Option<(&'a WindowDescriptor, &'a ShaderDescriptor, &'a str)>;
    fn is_render_update<'a>(&'a self) -> Option<(&'a WindowId, Option<&'a [I]>, Option<&'a [V]>)>;
}
