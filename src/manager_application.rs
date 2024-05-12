#![allow(deprecated)]
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

trait SurfaceWrapper {
    fn surface<'a, 'b: 'a>(&'b self) -> &'a wgpu::Surface<'a>;
}
struct Surface<'a> {
    surface: wgpu::Surface<'a>,
}
impl<'a> SurfaceWrapper for Surface<'a> {
    fn surface<'b, 'c: 'b>(&'c self) -> &'b wgpu::Surface<'b> {
        &self.surface
    }
}
pub struct GraphicsProvider {
    instance: wgpu::Instance,
    adapter: Option<wgpu::Adapter>,
    surfaces: Vec<(WindowId, Box<dyn SurfaceWrapper>)>,
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
            surfaces: Vec::new(),
        }
    }

    fn init(&mut self, window: &Window) {
        self.init_window(window);
        let surface = self.surfaces.last().unwrap().1.surface();
        let adapter = futures::executor::block_on(self.instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ))
        .unwrap();
        self.adapter = Some(adapter);
    }

    fn init_window(&mut self, window: &Window) {
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
        self.surfaces
            .push((window.id(), Box::new(Surface { surface })));
    }

    fn remove_window(&mut self, id: WindowId) {
        self.surfaces.retain(|(i, _)| *i != id);
    }
}

pub struct ManagerApplication<E: 'static, M: EventManager<E>> {
    event_manager: M,
    window_manager: WindowManager<E>,
    graphics_provider: GraphicsProvider,
}

impl<'a, E: 'static, M: EventManager<E>> ApplicationHandler<E> for ManagerApplication<E, M> {
    fn resumed(&mut self, active_loop: &ActiveEventLoop) {
        let window = active_loop
            .create_window(
                self.window_manager
                    .default_window
                    .get_attributes(active_loop),
            )
            .unwrap();
        self.window_manager.windows = vec![window];
        self.graphics_provider.init(&self.window_manager.windows[0]);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                if self.window_manager.windows.len() == 1 {
                    event_loop.exit();
                } else {
                    self.graphics_provider.remove_window(id);
                    self.window_manager.windows.retain(|w| w.id() != id);
                }
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

    pub fn run(&mut self) {
        let event_loop = EventLoop::<E>::with_user_event().build().unwrap();
        let event_loop_proxy = event_loop.create_proxy();
        self.window_manager.event_loop = Some(event_loop_proxy);

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(self).unwrap();
    }
}
