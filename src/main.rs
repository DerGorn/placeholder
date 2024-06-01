use env_logger::Env;
use placeholder::app::{ManagerApplication, WindowDescriptor};
use placeholder::graphics::ShaderDescriptor;
use std::fmt::Debug;
use std::path::PathBuf;
use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowAttributes,
};

mod vertex;
use vertex::Vertex;

mod game;
use game::{
    Entity, Game, Index, RessourceDescriptor, Scene, SpriteDescriptor, SpritePosition, SpriteSheet,
    SpriteSheetDimensions, SpriteSheetName,
};
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view: [[f32; 3]; 2],
}
impl CameraUniform {
    fn new() -> Self {
        Self {
            view: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        }
    }
}
impl From<&Camera> for CameraUniform {
    fn from(camera: &Camera) -> Self {
        let c = Self {
            view: [
                [
                    2.0 / camera.view_size.width,
                    0.0,
                    -2.0 * camera.position.x / camera.view_size.width,
                ],
                [
                    0.0,
                    2.0 / camera.view_size.height,
                    -2.0 * camera.position.y / camera.view_size.height,
                ],
            ],
        };
        println!("{:?}", c);
        c
    }
}
#[derive(Clone)]
struct CameraDescriptor {
    position: Vector<f32>,
    view_size: PhysicalSize<f32>,
    speed: f32,
}
impl From<&CameraDescriptor> for Camera {
    fn from(descriptor: &CameraDescriptor) -> Self {
        Self::new(
            descriptor.position.clone(),
            descriptor.view_size,
            descriptor.speed,
        )
    }
}
struct Camera {
    position: Vector<f32>,
    velocity: Vector<f32>,
    max_speed: f32,
    acceleration: VelocityController,
    view_size: PhysicalSize<f32>,
}
impl Camera {
    fn new(position: Vector<f32>, view_size: PhysicalSize<f32>, speed: f32) -> Self {
        Self {
            position,
            velocity: Vector::new(0.0, 0.0, 0.0),
            max_speed: speed.powi(2),
            acceleration: VelocityController::new(speed / 10.0),
            view_size,
        }
    }
    fn update(&mut self) {
        let acceleration = self.acceleration.get_velocity();
        if acceleration == Vector::new(0.0, 0.0, 0.0) {
            self.velocity *= 0.9;
        } else {
            self.velocity += acceleration;
        }
        if self.velocity.magnitude_squared() > self.max_speed {
            self.velocity = self.velocity.normalize() * self.max_speed;
        }
        self.position += &self.velocity;
    }
    fn handle_key_input(&mut self, input: &KeyEvent) {
        if input.state == winit::event::ElementState::Released {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.acceleration.set_direction(Direction::Up, false);
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.acceleration.set_direction(Direction::Left, false);
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.acceleration.set_direction(Direction::Right, false);
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.acceleration.set_direction(Direction::Down, false);
                }
                _ => {}
            }
        } else if input.state == winit::event::ElementState::Pressed {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.acceleration.set_direction(Direction::Up, true);
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.acceleration.set_direction(Direction::Left, true);
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.acceleration.set_direction(Direction::Right, true);
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.acceleration.set_direction(Direction::Down, true);
                }
                _ => {}
            }
        }
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
        if magnitude >= 1.0 {
            velocity *= 1.0 / magnitude.sqrt();
        }
        velocity * self.speed
    }
}
struct Background {
    sprite_sheet: SpriteSheetName,
}
impl Debug for Background {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Background")
            .field("z", &self.z())
            .field("sprite", &self.sprite_sheet())
            .finish()
    }
}
impl Entity for Background {
    fn update(&mut self) {}
    fn sprite_sheet(&self) -> &SpriteSheetName {
        &self.sprite_sheet
    }
    fn z(&self) -> f32 {
        -1000.0
    }
    fn handle_key_input(&mut self, _input: &KeyEvent) {}
    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        window_size: &PhysicalSize<u32>,
        sprite_sheet: &SpriteSheet,
    ) {
        let x = 0.0;
        let y = 0.0;
        let z = 0.0;
        let x_offset = 1280.0;
        let y_offset = 800.0;
        let texture_coords = sprite_sheet.get_sprite_coordinates(&SpritePosition::new(0, 0));
        let new_vertices = [
            Vertex::new(
                Vector::new(x - x_offset, y + y_offset, 0.0),
                &texture_coords[0],
                sprite_sheet.texture(),
            ),
            Vertex::new(
                Vector::new(x + x_offset, y + y_offset, 0.0),
                &texture_coords[1],
                sprite_sheet.texture(),
            ),
            Vertex::new(
                Vector::new(x + x_offset, y - y_offset, 0.0),
                &texture_coords[2],
                sprite_sheet.texture(),
            ),
            Vertex::new(
                Vector::new(x - x_offset, y - y_offset, 0.0),
                &texture_coords[3],
                sprite_sheet.texture(),
            ),
        ];
        let start_index = vertices.len() as u16;
        let new_indices = [
            start_index,
            start_index + 1,
            start_index + 2,
            start_index,
            start_index + 2,
            start_index + 3,
        ];
        println!("Background_vertices: {:?}", new_vertices);
        vertices.extend_from_slice(&new_vertices);
        indices.extend_from_slice(&new_indices);
    }
}
struct Player {
    width: u16,
    position: Vector<f32>,
    velocity: VelocityController,
    sprite: SpriteDescriptor,
}
impl Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player")
            .field("z", &self.z())
            .field("sprite", &self.sprite_sheet())
            .finish()
    }
}
impl Entity for Player {
    fn update(&mut self) {
        self.position += self.velocity.get_velocity();
    }

    fn sprite_sheet(&self) -> &SpriteSheetName {
        &self.sprite.sprite_sheet()
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
        let x_offset = self.width as f32;
        let y_offset = self.width as f32;
        let texture_coords = sprite_sheet.get_sprite_coordinates(&self.sprite.position);
        let new_vertices = [
            Vertex::new(
                Vector::new(x - x_offset, y + y_offset, 0.0),
                &texture_coords[0],
                sprite_sheet.texture(),
            ),
            Vertex::new(
                Vector::new(x + x_offset, y + y_offset, 0.0),
                &texture_coords[1],
                sprite_sheet.texture(),
            ),
            Vertex::new(
                Vector::new(x + x_offset, y - y_offset, 0.0),
                &texture_coords[2],
                sprite_sheet.texture(),
            ),
            Vertex::new(
                Vector::new(x - x_offset, y - y_offset, 0.0),
                &texture_coords[3],
                sprite_sheet.texture(),
            ),
        ];
        let start_index = vertices.len() as u16;
        let new_indices = [
            start_index,
            start_index + 1,
            start_index + 2,
            start_index,
            start_index + 2,
            start_index + 3,
        ];
        println!("Player_vertices: {:?}", new_vertices);
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

const PLAYER_NEUTRAL: SpritePosition = SpritePosition::new(0, 0);
const PLAYER_DOWN: SpritePosition = SpritePosition::new(1, 0);
const PLAYER_UP: SpritePosition = SpritePosition::new(2, 0);
const PLAYER_LEFT: SpritePosition = SpritePosition::new(3, 0);
const PLAYER_RIGHT: SpritePosition = SpritePosition::new(4, 0);

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let target_fps = 60;

    let cursor_path = "res/images/cursor/Cursor_Goth_Cursor.png";
    let main_window = WindowAttributes::default().with_title("Wispers in the Void - Dark Dynasty");
    let main_window_descriptor = WindowDescriptor::new(main_window).with_cursor(cursor_path);
    let shader_descriptor = ShaderDescriptor {
        file: "res/shader/shader.wgsl",
        vertex_shader: "vs_main",
        fragment_shader: "fs_main",
    };
    let camera_descriptor = CameraDescriptor {
        position: Vector::new(0.0, 0.0, 1.0),
        view_size: PhysicalSize::new(400.0, 400.0),
        speed: 0.01,
    };
    let main_window = "MainWindow";
    let player_sprite_sheet = "PlayerSpriteSheet";
    let background = "Background";
    let ressources = RessourceDescriptor {
        windows: vec![(
            main_window.into(),
            main_window_descriptor,
            shader_descriptor,
            camera_descriptor,
        )],
        sprite_sheets: vec![
            (
                player_sprite_sheet.into(),
                PathBuf::from("res/images/spriteSheets/protagonist.png"),
                SpriteSheetDimensions::new(8, 8),
            ),
            (
                background.into(),
                PathBuf::from("res/images/spriteSheets/background.png"),
                SpriteSheetDimensions::new(1, 1),
            ),
        ],
    };
    let scene = Scene {
        target_window: main_window.into(),
        entities: vec![
            Box::new(Player {
                width: 150,
                position: Vector::new(0.0, 0.0, 0.0),
                velocity: VelocityController::new(0.01),
                sprite: SpriteDescriptor::new(player_sprite_sheet.into(), PLAYER_NEUTRAL),
            }),
            Box::new(Background {
                sprite_sheet: background.into(),
            }),
        ],
    };
    let mut app = ManagerApplication::new(Game::new(ressources, vec![scene], target_fps));
    app.run();
}
