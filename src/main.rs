use std::{
    thread,
    time::{Duration, Instant},
};

use placeholder::app::{
    ApplicationEvent, EventManager, ManagerApplication, WindowDescriptor, WindowManager,
};
use placeholder::graphics::{ShaderDescriptor, Vertex as Vert};
use repr_trait::C;
use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    event::{KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::{WindowAttributes, WindowId},
};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    texture: u32,
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
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Uint32,
                    shader_location: 2,
                },
            ],
        }
    }
}

enum PlayerDirection {
    Neutral,
    Up,
    Down,
    Left,
    Right,
}
struct Square {
    width: u16,
    position: Vector<f32>,
    texture: u32,
    texture_coords: [f32; 8],
}
impl Square {
    fn render(&self, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, size: &PhysicalSize<u32>) {
        let x = self.position.x;
        let y = self.position.y;
        let z = self.position.z;
        let x_offset = self.width as f32 / (2.0 * size.width as f32);
        let y_offset = self.width as f32 / (2.0 * size.height as f32);
        let new_vertices = [
            Vertex {
                position: [x - x_offset, y + y_offset, z],
                tex_coords: [self.texture_coords[0], self.texture_coords[1]],
                texture: self.texture,
            },
            Vertex {
                position: [x + x_offset, y + y_offset, z],
                tex_coords: [self.texture_coords[2], self.texture_coords[3]],
                texture: self.texture,
            },
            Vertex {
                position: [x + x_offset, y - y_offset, z],
                tex_coords: [self.texture_coords[4], self.texture_coords[5]],
                texture: self.texture,
            },
            Vertex {
                position: [x - x_offset, y - y_offset, z],
                tex_coords: [self.texture_coords[6], self.texture_coords[7]],
                texture: self.texture,
            },
        ];
        let new_indices = [0, 1, 2, 0, 2, 3];
        vertices.extend_from_slice(&new_vertices);
        indices.extend_from_slice(&new_indices);
    }

    fn handle_key_input(&mut self, input: &KeyEvent) -> PlayerDirection {
        if input.state == winit::event::ElementState::Pressed {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.position.y += 0.01;
                    PlayerDirection::Up
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.position.x -= 0.01;
                    PlayerDirection::Left
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.position.x += 0.01;
                    PlayerDirection::Right
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.position.y -= 0.01;
                    PlayerDirection::Down
                }
                _ => PlayerDirection::Neutral,
            }
        } else {
            PlayerDirection::Neutral
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct WindowName(String);
impl WindowName {
    fn as_str<'a>(&'a self) -> &'a str {
        self.0.as_str()
    }
}
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
    NewTexture(String, Option<u32>),
    RequestNewTexture(String, String),
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

    fn is_request_new_texture<'a>(&'a self) -> Option<(&'a str, &'a str)> {
        if let Self::RequestNewTexture(path, label) = self {
            Some((path, label))
        } else {
            None
        }
    }

    fn new_texture(label: &str, id: Option<u32>) -> Self {
        Self::NewTexture(label.to_string(), id)
    }

    fn new_window(id: &WindowId, name: &str) -> Self {
        Self::NewWindow(id.clone(), name.into())
    }
}

const MAIN_WINDOW: &str = "Main";
const PLAYER_NEUTRAL: &str = "PlayerNeutral";
const PLAYER_NEUTRAL_PATH: &str = "res/images/standing/neutral.png";
const PLAYER_DOWN: &str = "PlayerDown";
const PLAYER_DOWN_PATH: &str = "res/images/forward/down.png";
const PLAYER_UP: &str = "PlayerUp";
const PLAYER_UP_PATH: &str = "res/images/back/up.png";
const PLAYER_LEFT: &str = "PlayerLeft";
const PLAYER_LEFT_PATH: &str = "res/images/left/left.png";
const PLAYER_RIGHT: &str = "PlayerRight";
const PLAYER_RIGHT_PATH: &str = "res/images/right/right.png";

struct EventHandler {
    default_window: WindowDescriptor,
    window_ids: Vec<(WindowName, WindowId)>,
    window_sizes: Vec<(WindowId, PhysicalSize<u32>)>,
    entities: Vec<(WindowId, Square)>,
    texture_ids: Vec<(String, u32)>,
    target_fps: u8,
}
impl EventHandler {
    fn new(default_window: WindowDescriptor, target_fps: u8) -> Self {
        Self {
            default_window,
            window_ids: Vec::new(),
            window_sizes: Vec::new(),
            entities: Vec::new(),
            texture_ids: Vec::new(),
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
        id: &winit::window::WindowId,
        event: &winit::event::WindowEvent,
    ) -> bool
    where
        Self: Sized,
    {
        match event {
            WindowEvent::Resized(size) => {
                let window_size = self.window_sizes.iter_mut().find(|(i, _)| i == id);
                if let Some((_, s)) = window_size {
                    *s = *size
                } else {
                    self.window_sizes.push((id.clone(), *size));
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.entities
                    .iter_mut()
                    .filter(|(i, _)| i == id)
                    .for_each(|(_, entity)| match entity.handle_key_input(event) {
                        PlayerDirection::Up => {
                            entity.texture = self
                                .texture_ids
                                .iter()
                                .find(|(n, _)| n == &PLAYER_UP)
                                .map(|(_, id)| *id)
                                .unwrap();
                        }
                        PlayerDirection::Down => {
                            entity.texture = self
                                .texture_ids
                                .iter()
                                .find(|(n, _)| n == &PLAYER_DOWN)
                                .map(|(_, id)| *id)
                                .unwrap();
                        }
                        PlayerDirection::Left => {
                            entity.texture = self
                                .texture_ids
                                .iter()
                                .find(|(n, _)| n == &PLAYER_LEFT)
                                .map(|(_, id)| *id)
                                .unwrap();
                        }
                        PlayerDirection::Right => {
                            entity.texture = self
                                .texture_ids
                                .iter()
                                .find(|(n, _)| n == &PLAYER_RIGHT)
                                .map(|(_, id)| *id)
                                .unwrap();
                        }
                        PlayerDirection::Neutral => {
                            entity.texture = self
                                .texture_ids
                                .iter()
                                .find(|(n, _)| n == &PLAYER_NEUTRAL)
                                .map(|(_, id)| *id)
                                .unwrap();
                        }
                    });
            }
            _ => {}
        }
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
                        MAIN_WINDOW.into(),
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
            Event::NewWindow(id, name) => {
                self.window_ids.push((name.clone(), id.clone()));
                window_manager
                    .send_event(Event::RequestNewTexture(
                        PLAYER_NEUTRAL_PATH.to_string(),
                        PLAYER_NEUTRAL.to_string(),
                    ))
                    .unwrap();
                window_manager
                    .send_event(Event::RequestNewTexture(
                        PLAYER_DOWN_PATH.to_string(),
                        PLAYER_DOWN.to_string(),
                    ))
                    .unwrap();
                window_manager
                    .send_event(Event::RequestNewTexture(
                        PLAYER_UP_PATH.to_string(),
                        PLAYER_UP.to_string(),
                    ))
                    .unwrap();
                window_manager
                    .send_event(Event::RequestNewTexture(
                        PLAYER_LEFT_PATH.to_string(),
                        PLAYER_LEFT.to_string(),
                    ))
                    .unwrap();
                window_manager
                    .send_event(Event::RequestNewTexture(
                        PLAYER_RIGHT_PATH.to_string(),
                        PLAYER_RIGHT.to_string(),
                    ))
                    .unwrap();
            }
            Event::NewTexture(label, None) => {
                if label.as_str() == PLAYER_NEUTRAL {
                    window_manager
                        .send_event(Event::RequestNewTexture(
                            PLAYER_NEUTRAL_PATH.to_string(),
                            PLAYER_NEUTRAL.to_string(),
                        ))
                        .unwrap();
                }
                if label.as_str() == PLAYER_DOWN {
                    window_manager
                        .send_event(Event::RequestNewTexture(
                            PLAYER_DOWN_PATH.to_string(),
                            PLAYER_DOWN.to_string(),
                        ))
                        .unwrap();
                }
                if label.as_str() == PLAYER_UP {
                    window_manager
                        .send_event(Event::RequestNewTexture(
                            PLAYER_UP_PATH.to_string(),
                            PLAYER_UP.to_string(),
                        ))
                        .unwrap();
                }
                if label.as_str() == PLAYER_LEFT {
                    window_manager
                        .send_event(Event::RequestNewTexture(
                            PLAYER_LEFT_PATH.to_string(),
                            PLAYER_LEFT.to_string(),
                        ))
                        .unwrap();
                }
                if label.as_str() == PLAYER_RIGHT {
                    window_manager
                        .send_event(Event::RequestNewTexture(
                            PLAYER_RIGHT_PATH.to_string(),
                            PLAYER_RIGHT.to_string(),
                        ))
                        .unwrap();
                }
            }
            Event::NewTexture(label, Some(id)) => {
                self.texture_ids.push((label.clone(), id.clone()));
                if label.as_str() == PLAYER_NEUTRAL {
                    self.entities.push((
                        self.get_window_id(MAIN_WINDOW.into()).unwrap().clone(),
                        Square {
                            width: 200,
                            position: Vector::new(0.0, 0.0, 0.0),
                            texture: *id,
                            texture_coords: [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0],
                        },
                    ));
                }
            }
            Event::Timer(_delta_t) => {
                for (_name, id) in &self.window_ids {
                    let size = self
                        .window_sizes
                        .iter()
                        .find(|(i, _)| i == id)
                        .map(|(_, s)| *s)
                        .or_else(|| Some(PhysicalSize::new(1, 1)))
                        .unwrap();
                    let mut vertices = Vec::new();
                    let mut indices = Vec::new();
                    for (target_id, square) in &self.entities {
                        if target_id != id {
                            continue;
                        }
                        square.render(&mut vertices, &mut indices, &size);
                    }
                    window_manager
                        .send_event(Event::RenderUpdate(*id, vertices, indices))
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
