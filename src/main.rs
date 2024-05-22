use std::{
    thread,
    time::{Duration, Instant},
};

use placeholder::app::{
    ApplicationEvent, EventManager, ManagerApplication, WindowDescriptor, WindowManager,
};
use placeholder::graphics::{ShaderDescriptor, Vertex as Vert};
use repr_trait::C;
use winit::window::{WindowAttributes, WindowId};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
impl Vert for Vertex {
    fn describe_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 1,
                },
            ],
        }
    }
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    },
];
const MAIN_INDICES: &[u16] = &[0, 1, 4, 1, 2, 4];
const SECOND_INDICES: &[u16] = &[1, 2, 4, 2, 3, 4];

#[derive(Debug, Clone, PartialEq)]
struct WindowName(String);
impl From<&str> for WindowName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
#[derive(Debug)]
enum Event {
    Timer(Duration),
    Resumed,
    NewWindow(WindowId, WindowName),
    RequestNewWindow(WindowDescriptor, ShaderDescriptor, WindowName),
    RenderUpdate(WindowId, Vec<Vertex>, Vec<u16>),
}
impl ApplicationEvent<u16, Vertex> for Event {
    fn app_resumed() -> Self {
        Self::Resumed
    }

    fn is_request_new_window<'a>(
        &'a self,
    ) -> Option<(&'a WindowDescriptor, &'a ShaderDescriptor, &'a str)> {
        if let Self::RequestNewWindow(window_descriptor, shader_descriptor, name) = self {
            Some((&window_descriptor, &shader_descriptor, name.0.as_str()))
        } else {
            None
        }
    }

    fn is_render_update<'a>(
        &'a self,
    ) -> Option<(
        &'a winit::window::WindowId,
        Option<&'a [u16]>,
        Option<&'a [Vertex]>,
    )> {
        if let Self::RenderUpdate(id, vertices, indices) = self {
            Some((
                &id,
                if vertices.len() > 0 {
                    Some(indices.as_slice())
                } else {
                    None
                },
                if indices.len() > 0 {
                    Some(vertices.as_slice())
                } else {
                    None
                },
            ))
        } else {
            None
        }
    }

    fn new_window(id: &WindowId, name: &str) -> Self {
        Self::NewWindow(id.clone(), name.into())
    }
}
struct EventHandler {
    default_window: WindowDescriptor,
    window_ids: Vec<(WindowName, WindowId)>,
    target_fps: u8,
}
impl EventHandler {
    fn new(default_window: WindowDescriptor, target_fps: u8) -> Self {
        Self {
            default_window,
            window_ids: Vec::new(),
            target_fps,
        }
    }

    fn get_window_id(&self, name: WindowName) -> Option<&WindowId> {
        self.window_ids
            .iter()
            .find(|(n, _)| n == &name)
            .map(|(_, id)| id)
    }
}
impl EventManager<Event> for EventHandler {
    fn window_event(
        &mut self,
        _window_manager: &mut WindowManager<Event>,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _id: &winit::window::WindowId,
        _event: &winit::event::WindowEvent,
    ) -> bool
    where
        Self: Sized,
    {
        // todo!()
        true
    }

    fn user_event(
        &mut self,
        window_manager: &mut WindowManager<Event>,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: &Event,
    ) where
        Self: Sized,
    {
        match event {
            Event::Resumed => {
                let descriptor = self.default_window.clone();
                let shader_descriptor = ShaderDescriptor {
                    file: "res/shader/shader.wgsl",
                    vertex_shader: "vs_main",
                    fragment_shader: "fs_main",
                };
                window_manager
                    .send_event(Event::RequestNewWindow(
                        descriptor.clone(),
                        shader_descriptor.clone(),
                        "Main".into(),
                    ))
                    .unwrap();
                window_manager
                    .send_event(Event::RequestNewWindow(
                        descriptor,
                        shader_descriptor,
                        "Secondary".into(),
                    ))
                    .unwrap();
                let ns_per_frame = 1e9 / (self.target_fps as f64);
                let frame_duration = Duration::from_nanos(ns_per_frame as u64);
                let timer_event_loop = window_manager.create_event_loop_proxy();
                thread::spawn(move || {
                    let mut last_update = Instant::now();
                    loop {
                        match timer_event_loop.send_event(Event::Timer(last_update.elapsed())) {
                            Ok(()) => {}
                            Err(_) => break,
                        };
                        last_update = Instant::now();
                        thread::sleep(frame_duration);
                    }
                });
            }
            Event::NewWindow(id, name) => self.window_ids.push((name.clone(), id.clone())),
            Event::Timer(delta_t) => {
                if let Some(main_id) = self.get_window_id("Main".into()) {
                    window_manager
                        .send_event(Event::RenderUpdate(
                            main_id.clone(),
                            VERTICES.to_vec(),
                            MAIN_INDICES.to_vec(),
                        ))
                        .unwrap();
                }
                if let Some(second_id) = self.get_window_id("Secondary".into()) {
                    window_manager
                        .send_event(Event::RenderUpdate(
                            second_id.clone(),
                            VERTICES.to_vec(),
                            SECOND_INDICES.to_vec(),
                        ))
                        .unwrap();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let target_fps = 60;

    let cursor_path = "res/images/cursor/Cursor_Goth_Cursor.png";
    let default_window =
        WindowAttributes::default().with_title("Wispers in the Void - Dark Dynasty");
    let default_window = WindowDescriptor::new(default_window).with_cursor(cursor_path);
    let mut app = ManagerApplication::new(EventHandler::new(default_window, target_fps));
    app.run();
}
