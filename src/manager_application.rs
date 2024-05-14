#![allow(deprecated)]
use std::fmt::Debug;
use wgpu::rwh::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

mod window_descriptor;
pub use window_descriptor::WindowDescriptor;

mod event_manager;
pub use event_manager::EventManager;

pub struct WindowManager<E: 'static> {
    windows: Vec<Window>,
    event_loop: Option<EventLoopProxy<E>>,
    default_window: WindowDescriptor,
}
impl<E: 'static> Default for WindowManager<E> {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            event_loop: None,
            default_window: Default::default(),
        }
    }
}

trait WindowSurface: Debug {
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
    fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let output = self.surface().get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
struct Surface<'a> {
    surface: wgpu::Surface<'a>,
    size: winit::dpi::PhysicalSize<u32>,
    config: wgpu::SurfaceConfiguration,
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
        &self.surface
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
}
pub struct GraphicsProvider {
    instance: wgpu::Instance,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surfaces: Vec<(WindowId, Box<dyn WindowSurface>)>,
}
impl GraphicsProvider {
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

    fn init_window(&mut self, window: &Window) {
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

        self.surfaces.push((
            window.id(),
            Box::new(Surface {
                surface,
                size,
                config,
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

    fn render_window(&self, id: &WindowId) {
        if let Some((_, surface)) = self.surfaces.iter().find(|(i, _)| i == id) {
            if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
                surface.render(device, queue);
            }
        }
    }

    fn remove_window(&mut self, id: &WindowId) {
        self.surfaces.retain(|(i, _)| i != id);
    }
}

pub struct ManagerApplication<E: 'static, M: EventManager<E>> {
    event_manager: M,
    window_manager: WindowManager<E>,
    graphics_provider: GraphicsProvider,
}

impl<'a, E: 'static, M: EventManager<E>> ApplicationHandler<E> for ManagerApplication<E, M> {
    fn resumed(&mut self, active_loop: &ActiveEventLoop) {
        let descriptor = self.window_manager.default_window.clone();
        self.create_window(&descriptor, active_loop);
        self.create_window(&descriptor, active_loop)
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                if self.window_manager.windows.len() == 1 {
                    event_loop.exit();
                } else {
                    self.graphics_provider.remove_window(&id);
                    self.window_manager.windows.retain(|w| w.id() != id);
                }
            }
            WindowEvent::Resized(size) => self.graphics_provider.resize_window(&id, &size),
            WindowEvent::ScaleFactorChanged { .. } => {
                //TODO: I think the window will be resized  on its own, which fires a Resized event
            }
            WindowEvent::RedrawRequested => {
                self.graphics_provider.render_window(&id);
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
                let window = self
                    .window_manager
                    .windows
                    .iter()
                    .find(|w| w.id() == id)
                    .unwrap();
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

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: E) {
        self.event_manager
            .user_event(&mut self.window_manager, event_loop, event);
    }
}

impl<'a, E: 'static, M: EventManager<E>> ManagerApplication<E, M> {
    pub fn new(event_manager: M, default_window: Option<WindowDescriptor>) -> Self {
        Self {
            event_manager,
            window_manager: WindowManager {
                default_window: default_window.unwrap_or_default(),
                ..Default::default()
            },
            graphics_provider: GraphicsProvider::new(),
        }
    }

    pub fn create_window(&mut self, descriptor: &WindowDescriptor, active_loop: &ActiveEventLoop) {
        let window = active_loop
            .create_window(descriptor.get_attributes(active_loop))
            .unwrap();
        self.window_manager.windows.push(window);
        self.graphics_provider
            .init_window(&self.window_manager.windows.last().unwrap());
    }

    pub fn run(&mut self) {
        env_logger::init();
        let event_loop = EventLoop::<E>::with_user_event().build().unwrap();
        let event_loop_proxy = event_loop.create_proxy();
        self.window_manager.event_loop = Some(event_loop_proxy);

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(self).unwrap();
    }
}
