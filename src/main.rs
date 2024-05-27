use std::{
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use placeholder::app::{
    ApplicationEvent, EventManager, ManagerApplication, WindowDescriptor, WindowManager,
};
use placeholder::graphics::ShaderDescriptor;
use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    event::{KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::{WindowAttributes, WindowId},
};
mod vertex;
use vertex::Vertex;

macro_rules! create_name_struct {
    ($name: ident) => {
        #[derive(Debug, Clone, PartialEq)]
        struct $name(String);
        impl $name {
            #[allow(dead_code)]
            fn as_str<'a>(&'a self) -> &'a str {
                self.0.as_str()
            }
        }
        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }
        impl From<String> for $name {
            fn from(value: String) -> Self {
                value.as_str().into()
            }
        }
        impl From<&String> for $name {
            fn from(value: &String) -> Self {
                value.as_str().into()
            }
        }
    };
}
create_name_struct!(SpriteSheetName);
create_name_struct!(WindowName);

struct SpritePosition {
    x: u8,
    y: u8,
}
impl SpritePosition {
    const fn new(x: u8, y: u8) -> Self {
        SpritePosition { x, y }
    }
}
struct SpriteDescriptor {
    sprite_sheet: SpriteSheetName,
    position: SpritePosition,
}

type Index = u16;
trait Entity {
    fn update(&mut self);
    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        window_size: &PhysicalSize<u32>,
        sprite_sheet: &SpriteSheet,
    );
    fn sprite_sheet(&self) -> &SpriteSheetName;
    fn handle_key_input(&mut self, input: &KeyEvent);
    fn z(&self) -> f32 {
        0.0
    }
}

enum Direction {
    Up,
    Right,
    Down,
    Left,
}
/// 8 directional VelocityController
struct VelocityController {
    speed: f32,
    up: bool,
    right: bool,
    down: bool,
    left: bool,
}
impl VelocityController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            up: false,
            right: false,
            down: false,
            left: false,
        }
    }

    fn set_direction(&mut self, direction: Direction, value: bool) {
        match direction {
            Direction::Up => {
                self.up = value;
            }
            Direction::Right => {
                self.right = value;
            }
            Direction::Down => {
                self.down = value;
            }
            Direction::Left => {
                self.left = value;
            }
        }
    }

    fn get_velocity(&self) -> Vector<f32> {
        let mut velocity = Vector::new(0.0, 0.0, 0.0);
        if self.up {
            velocity.y += 1.0;
        }
        if self.right {
            velocity.x += 1.0;
        }
        if self.down {
            velocity.y -= 1.0;
        }
        if self.left {
            velocity.x -= 1.0;
        }
        let magnitude: f32 = velocity.magnitude_squared();
        if magnitude != 0.0 {
            velocity *= 1.0 / magnitude.sqrt();
        }
        velocity * self.speed
    }
}
struct TextureCoordinates {
    u: f32,
    v: f32,
}
struct Square {
    width: u16,
    position: Vector<f32>,
    velocity: VelocityController,
    sprite: SpriteDescriptor,
}
impl Entity for Square {
    fn update(&mut self) {
        self.position += self.velocity.get_velocity();
    }

    fn sprite_sheet(&self) -> &SpriteSheetName {
        &self.sprite.sprite_sheet
    }

    fn z(&self) -> f32 {
        self.position.z
    }

    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        window_size: &PhysicalSize<u32>,
        sprite_sheet: &SpriteSheet,
    ) {
        let x = self.position.x;
        let y = self.position.y;
        let z = self.position.z;
        let x_offset = self.width as f32 / (window_size.width as f32);
        let y_offset = self.width as f32 / (window_size.height as f32);
        let texture_coords = sprite_sheet.get_sprite_coordinates(&self.sprite.position);
        let new_vertices = [
            Vertex::new(
                Vector::new(x - x_offset, y + y_offset, z),
                &texture_coords[0],
                sprite_sheet.texture,
            ),
            Vertex::new(
                Vector::new(x + x_offset, y + y_offset, z),
                &texture_coords[1],
                sprite_sheet.texture,
            ),
            Vertex::new(
                Vector::new(x + x_offset, y - y_offset, z),
                &texture_coords[2],
                sprite_sheet.texture,
            ),
            Vertex::new(
                Vector::new(x - x_offset, y - y_offset, z),
                &texture_coords[3],
                sprite_sheet.texture,
            ),
        ];
        let new_indices = [0, 1, 2, 0, 2, 3];
        vertices.extend_from_slice(&new_vertices);
        indices.extend_from_slice(&new_indices);
    }

    fn handle_key_input(&mut self, input: &KeyEvent) {
        if input.state == winit::event::ElementState::Released {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.velocity.set_direction(Direction::Up, false);
                    self.sprite.position = PLAYER_NEUTRAL;
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.velocity.set_direction(Direction::Left, false);
                    self.sprite.position = PLAYER_NEUTRAL;
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.velocity.set_direction(Direction::Right, false);
                    self.sprite.position = PLAYER_NEUTRAL;
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.velocity.set_direction(Direction::Down, false);
                    self.sprite.position = PLAYER_NEUTRAL;
                }
                _ => {}
            }
        } else if input.state == winit::event::ElementState::Pressed {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.velocity.set_direction(Direction::Up, true);
                    self.sprite.position = PLAYER_UP;
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.velocity.set_direction(Direction::Left, true);
                    self.sprite.position = PLAYER_LEFT;
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.velocity.set_direction(Direction::Right, true);
                    self.sprite.position = PLAYER_RIGHT;
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.velocity.set_direction(Direction::Down, true);
                    self.sprite.position = PLAYER_DOWN;
                }
                _ => {}
            }
        } else {
            self.sprite.position = PLAYER_NEUTRAL;
        }
    }
}

struct SpriteSheet {
    texture: u32,
    sprites_per_row: u8,
    sprites_per_column: u8,
}
impl SpriteSheet {
    fn get_sprite_coordinates(&self, position: &SpritePosition) -> [TextureCoordinates; 4] {
        let width = 1.0 / self.sprites_per_row as f32;
        let height = 1.0 / self.sprites_per_column as f32;
        let x_offset = position.x as f32 * width;
        let y_offset = position.y as f32 * height;
        [
            TextureCoordinates {
                u: x_offset,
                v: y_offset,
            },
            TextureCoordinates {
                u: x_offset + width,
                v: y_offset,
            },
            TextureCoordinates {
                u: x_offset + width,
                v: y_offset + height,
            },
            TextureCoordinates {
                u: x_offset,
                v: y_offset + height,
            },
        ]
    }
}

#[derive(Debug)]
enum Event {
    Timer(Duration),
    Resumed,
    NewWindow(WindowId, WindowName),
    RequestNewWindow(WindowDescriptor, ShaderDescriptor, WindowName),
    RenderUpdate(WindowId, Vec<Vertex>, Vec<Index>),
    NewSpriteSheet(SpriteSheetName, Option<u32>),
    RequestNewSpriteSheet(SpriteSheetName, PathBuf),
}
impl ApplicationEvent<Index, Vertex> for Event {
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
        Option<&'a [Index]>,
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

    fn is_request_new_texture<'a>(&'a self) -> Option<(&'a Path, &'a str)> {
        if let Self::RequestNewSpriteSheet(label, path) = self {
            Some((path, label.as_str()))
        } else {
            None
        }
    }

    fn new_texture(label: &str, id: Option<u32>) -> Self {
        Self::NewSpriteSheet(label.into(), id)
    }

    fn new_window(id: &WindowId, name: &str) -> Self {
        Self::NewWindow(id.clone(), name.into())
    }
}

const MAIN_WINDOW: &str = "Main";
const PLAYER_SPRITE_SHEET: &str = "PlayerSpriteSheet";
const PLAYER_SPRITE_SHEET_PATH: &str = "res/images/spriteSheets/protagonist.png";
const PLAYER_SPRITE_SHEET_WIDTH: u8 = 8;
const PLAYER_NEUTRAL: SpritePosition = SpritePosition::new(0, 0);
const PLAYER_DOWN: SpritePosition = SpritePosition::new(1, 0);
const PLAYER_UP: SpritePosition = SpritePosition::new(2, 0);
const PLAYER_LEFT: SpritePosition = SpritePosition::new(3, 0);
const PLAYER_RIGHT: SpritePosition = SpritePosition::new(4, 0);

struct EventHandler {
    default_window: WindowDescriptor,
    window_ids: Vec<(WindowName, WindowId)>,
    window_sizes: Vec<(WindowId, PhysicalSize<u32>)>,
    entities: Vec<(WindowId, Box<dyn Entity>)>,
    sprite_sheets: Vec<(String, SpriteSheet)>,
    target_fps: u8,
}
impl EventHandler {
    fn new(default_window: WindowDescriptor, target_fps: u8) -> Self {
        Self {
            default_window,
            window_ids: Vec::new(),
            window_sizes: Vec::new(),
            entities: Vec::new(),
            sprite_sheets: Vec::new(),
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
                    .for_each(|(_, entity)| entity.handle_key_input(event));
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
                    .send_event(Event::RequestNewSpriteSheet(
                        PLAYER_SPRITE_SHEET.into(),
                        PLAYER_SPRITE_SHEET_PATH.into(),
                    ))
                    .unwrap();
            }
            Event::NewSpriteSheet(label, None) => {
                if label.as_str() == PLAYER_SPRITE_SHEET {
                    window_manager
                        .send_event(Event::RequestNewSpriteSheet(
                            PLAYER_SPRITE_SHEET.into(),
                            PLAYER_SPRITE_SHEET_PATH.into(),
                        ))
                        .unwrap();
                }
            }
            Event::NewSpriteSheet(label, Some(id)) => {
                if label.as_str() == PLAYER_SPRITE_SHEET {
                    let sprite_sheet = SpriteSheet {
                        texture: *id,
                        sprites_per_row: PLAYER_SPRITE_SHEET_WIDTH,
                        sprites_per_column: PLAYER_SPRITE_SHEET_WIDTH,
                    };
                    self.entities.push((
                        self.get_window_id(MAIN_WINDOW.into()).unwrap().clone(),
                        Box::new(Square {
                            width: 150,
                            position: Vector::new(0.0, 0.0, 0.0),
                            velocity: VelocityController::new(0.01),
                            sprite: SpriteDescriptor {
                                sprite_sheet: label.clone(),
                                position: PLAYER_NEUTRAL,
                            },
                        }),
                    ));
                    self.sprite_sheets
                        .push((label.as_str().to_string(), sprite_sheet));
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
                    self.entities
                        .sort_by(|(_, a), (_, b)| a.z().partial_cmp(&b.z()).unwrap());
                    for (target_id, entity) in self.entities.iter_mut() {
                        if target_id != id {
                            continue;
                        }
                        entity.update();
                        entity.render(
                            &mut vertices,
                            &mut indices,
                            &size,
                            &self
                                .sprite_sheets
                                .iter()
                                .find(|(l, _)| l == &entity.sprite_sheet().0)
                                .unwrap()
                                .1,
                        );
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
