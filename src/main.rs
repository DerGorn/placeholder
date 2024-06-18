use env_logger::Env;
use placeholder::app::{IndexBuffer, ManagerApplication, VertexBuffer, WindowDescriptor};
use placeholder::graphics::{RenderSceneDescriptor, ShaderDescriptor};
use std::fmt::Debug;
use std::path::PathBuf;
use std::time::Duration;
use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowAttributes,
};

use placeholder::graphics::{Index as I, Vertex as V};

use placeholder::game_engine::{
    BoundingBox, CameraDescriptor, Direction, Entity, EntityName, EntityType, ExternalEvent, Game,
    Index, RessourceDescriptor, Scene, SceneName, SpritePosition, SpriteSheet,
    SpriteSheetDimensions, SpriteSheetName, VelocityController,
};

mod animation;
use animation::Animation;

mod background;
use background::Background;

mod vertex;
use vertex::{render_sprite, Vertex};

#[derive(Debug, PartialEq)]
enum Type {
    Background,
    Player,
    Enemy,
}
impl EntityType for Type {}

#[derive(Debug)]
enum Event {
    RequestNewScenes(Vec<Scene<Self>>),
    NewScene(SceneName),
}
impl ExternalEvent for Event {
    type EntityType = Type;
    fn is_request_new_scenes<'a>(&'a self) -> bool {
        match self {
            Event::RequestNewScenes(_) => true,
            _ => false,
        }
    }

    fn consume_scenes_request(self) -> Option<Vec<Scene<Self>>>
    where
        Self: Sized,
    {
        match self {
            Event::RequestNewScenes(scenes) => Some(scenes),
            _ => None,
        }
    }

    fn new_scene(scene: &Scene<Self>) -> Self
    where
        Self: Sized,
    {
        Self::NewScene(scene.name.clone())
    }
}

use repr_trait::C;
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
struct SimpleVertex {
    position: [f32; 2],
}
impl SimpleVertex {
    fn new(position: Vector<f32>) -> Self {
        Self {
            position: [position.x, position.y],
        }
    }
}
impl V for SimpleVertex {
    fn describe_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                format: wgpu::VertexFormat::Float32x2,
                shader_location: 0,
            }],
        }
    }
}

struct Transition {
    name: EntityName,
    animation: Animation<(Vec<SimpleVertex>, Vec<Index>)>,
}
impl Debug for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transition")
            .field("name", &self.name)
            .finish()
    }
}
impl Entity<Type, Event> for Transition {
    fn render(
        &self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        _sprite_sheet: Option<&SpriteSheet>,
    ) {
        let (new_vertices, new_indices) = self.animation.keyframe();
        let start_index = vertices.len() as u16;
        vertices.extend_from_slice(new_vertices);
        indices.extend_from_slice(
            new_indices
                .iter()
                .map(|i| i + start_index)
                .collect::<Vec<_>>()
                .as_slice(),
        );
    }

    fn update(
        &mut self,
        _entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &Duration,
    ) -> Vec<Event> {
        self.animation.update(delta_t);
        vec![]
    }

    fn name(&self) -> &EntityName {
        &self.name
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: Vector::scalar(0.0),
            size: PhysicalSize::new(1e5, 1e5),
        }
    }

    fn entity_type(&self) -> Type {
        Type::Background
    }
}

struct Enemy {
    name: EntityName,
    size: PhysicalSize<u16>,
    position: Vector<f32>,
    animation: Animation<SpritePosition>,
}
impl Debug for Enemy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enemy")
            .field("z", &self.z())
            .field("sprite", &self.sprite_sheet())
            .finish()
    }
}
impl Entity<Type, Event> for Enemy {
    fn update(
        &mut self,
        entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &Duration,
    ) -> Vec<Event> {
        self.animation.update(delta_t);
        let players = entities.iter().filter(|e| e.entity_type() == Type::Player);
        let own_bounding_box = self.bounding_box();
        for player in players {
            let bounding_box = player.bounding_box();
            if own_bounding_box.intersects(&bounding_box) {
                let shader_descriptor = ShaderDescriptor {
                    file: "res/shader/transition.wgsl",
                    vertex_shader: "vs_main",
                    fragment_shader: "fs_main",
                };
                return vec![Event::RequestNewScenes(vec![Scene {
                    render_scene_descriptor: RenderSceneDescriptor {
                        index_format: Index::index_format(),
                        vertex_buffer_layout: SimpleVertex::describe_buffer_layout(),
                        use_textures: false,
                    },
                    name: "BattleScene".into(),
                    render_scene: "BattleScene".into(),
                    target_window: MAIN_WINDOW.into(),
                    z_index: 1,
                    entities: vec![Box::new(Transition {
                        name: "BattleTransition".into(),
                        animation: Animation::new(
                            "BattleTransition".into(),
                            vec![
                                (
                                    Duration::from_millis(24),
                                    (
                                        vec![
                                            SimpleVertex::new(Vector::new(-0.5, 0.5, 0.0)),
                                            SimpleVertex::new(Vector::new(0.5, 0.5, 0.0)),
                                            SimpleVertex::new(Vector::new(0.5, -0.5, 0.0)),
                                            SimpleVertex::new(Vector::new(-0.5, -0.5, 0.0)),
                                        ],
                                        vec![0, 1, 2, 0, 2, 3],
                                    ),
                                ),
                                (
                                    Duration::from_millis(24),
                                    (
                                        vec![
                                            SimpleVertex::new(Vector::new(-0.75, 0.75, 0.0)),
                                            SimpleVertex::new(Vector::new(0.75, 0.75, 0.0)),
                                            SimpleVertex::new(Vector::new(0.75, -0.75, 0.0)),
                                            SimpleVertex::new(Vector::new(-0.75, -0.75, 0.0)),
                                        ],
                                        vec![0, 1, 2, 0, 2, 3],
                                    ),
                                ),
                                (
                                    Duration::from_millis(24),
                                    (
                                        vec![
                                            SimpleVertex::new(Vector::new(-1.0, 1.0, 0.0)),
                                            SimpleVertex::new(Vector::new(1.0, 1.0, 0.0)),
                                            SimpleVertex::new(Vector::new(1.0, -1.0, 0.0)),
                                            SimpleVertex::new(Vector::new(-1.0, -1.0, 0.0)),
                                        ],
                                        vec![0, 1, 2, 0, 2, 3],
                                    ),
                                ),
                                (
                                    Duration::from_millis(24),
                                    (
                                        vec![
                                            SimpleVertex::new(Vector::new(-0.75, 0.75, 0.0)),
                                            SimpleVertex::new(Vector::new(0.75, 0.75, 0.0)),
                                            SimpleVertex::new(Vector::new(0.75, -0.75, 0.0)),
                                            SimpleVertex::new(Vector::new(-0.75, -0.75, 0.0)),
                                        ],
                                        vec![0, 1, 2, 0, 2, 3],
                                    ),
                                ),
                            ],
                        ),
                    })],
                    shader_descriptor,
                }])];
            }
        }
        vec![]
    }
    fn render(
        &self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        sprite_sheet: Option<&SpriteSheet>,
    ) {
        if let Some(sprite_sheet) = sprite_sheet {
            render_sprite(
                &self.bounding_box(),
                vertices,
                indices,
                sprite_sheet,
                self.animation.keyframe(),
            );
        }
    }
    fn sprite_sheet(&self) -> Option<&SpriteSheetName> {
        Some(&self.animation.sprite_sheet())
    }
    fn handle_key_input(&mut self, _input: &KeyEvent) {}
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: self.position.clone(),
            size: PhysicalSize::new(self.size.width as f32, self.size.height as f32),
        }
    }
    fn entity_type(&self) -> Type {
        Type::Enemy
    }
}

struct Player {
    name: EntityName,
    size: PhysicalSize<u16>,
    position: Vector<f32>,
    velocity: VelocityController,
    animation: Animation<SpritePosition>,
}
impl Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player")
            .field("z", &self.z())
            .field("sprite", &self.sprite_sheet())
            .finish()
    }
}
impl Entity<Type, Event> for Player {
    fn entity_type(&self) -> Type {
        Type::Player
    }
    fn update(
        &mut self,
        entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &Duration,
    ) -> Vec<Event> {
        self.position += self.velocity.get_velocity();
        let background = entities
            .iter()
            .filter(|e| e.entity_type() == Type::Background)
            .next()
            .expect("No Background found to restrict Playermovement");
        if let Some(new_position) = background
            .bounding_box()
            .clamp_box_inside(&self.bounding_box())
        {
            self.position = new_position;
        }
        self.animation.update(delta_t);
        vec![]
    }

    fn name(&self) -> &EntityName {
        &self.name
    }

    fn position(&self) -> Vector<f32> {
        self.position.clone()
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: self.position.clone(),
            size: PhysicalSize::new(self.size.width as f32, self.size.height as f32),
        }
    }

    fn sprite_sheet(&self) -> Option<&SpriteSheetName> {
        Some(&self.animation.sprite_sheet())
    }

    fn z(&self) -> f32 {
        self.position.z
    }

    fn render(
        &self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        sprite_sheet: Option<&SpriteSheet>,
    ) {
        if let Some(sprite_sheet) = sprite_sheet {
            render_sprite(
                &self.bounding_box(),
                vertices,
                indices,
                sprite_sheet,
                self.animation.keyframe(),
            );
        }
    }

    fn handle_key_input(&mut self, input: &KeyEvent) {
        if input.state == winit::event::ElementState::Released {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.velocity.set_direction(Direction::Up, false);
                    // self.sprite.position = PLAYER_NEUTRAL;
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.velocity.set_direction(Direction::Left, false);
                    // self.sprite.position = PLAYER_NEUTRAL;
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.velocity.set_direction(Direction::Right, false);
                    // self.sprite.position = PLAYER_NEUTRAL;
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.velocity.set_direction(Direction::Down, false);
                    // self.sprite.position = PLAYER_NEUTRAL;
                }
                _ => {}
            }
        } else if input.state == winit::event::ElementState::Pressed {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.velocity.set_direction(Direction::Up, true);
                    // self.sprite.position = PLAYER_UP;
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.velocity.set_direction(Direction::Left, true);
                    // self.sprite.position = PLAYER_LEFT;
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.velocity.set_direction(Direction::Right, true);
                    // self.sprite.position = PLAYER_RIGHT;
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.velocity.set_direction(Direction::Down, true);
                    // self.sprite.position = PLAYER_DOWN;
                }
                _ => {}
            }
        } else {
            // self.sprite.position = PLAYER_NEUTRAL;
        }
    }
}

const MAIN_WINDOW: &str = "MainWindow";
const FROG: &str = "Frog";
fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let target_fps = 60;

    let cursor_path = "res/images/cursor/Cursor_Goth_Cursor.png";
    let main_window = WindowAttributes::default().with_title("Wispers in the Void - Dark Dynasty");
    let main_window_descriptor = WindowDescriptor::new(main_window).with_cursor(cursor_path);
    let shader_descriptor = ShaderDescriptor {
        file: "res/shader/texture_array.wgsl",
        vertex_shader: "vs_main",
        fragment_shader: "fs_main",
    };
    let protaginist_name = "Protagonist";
    let player_sprite_sheet = "PlayerSpriteSheet";
    let background = "Background";
    let camera_descriptor = CameraDescriptor {
        view_size: PhysicalSize::new(800.0, 600.0),
        speed: 90.0,
        acceleration_steps: 30,
        target_entity: protaginist_name.into(),
        bound_entity: Some(background.into()),
        max_offset_position: 100.0,
    };
    let main_scene = "MainScene";
    let ressources = RessourceDescriptor {
        windows: vec![(MAIN_WINDOW.into(), main_window_descriptor)],
        render_scenes: vec![(main_scene.into(), camera_descriptor)],
        sprite_sheets: vec![
            (
                player_sprite_sheet.into(),
                PathBuf::from("res/images/spriteSheets/ProtagonistP.png"),
                SpriteSheetDimensions::new(4, 1),
            ),
            (
                background.into(),
                PathBuf::from("res/images/spriteSheets/background.png"),
                SpriteSheetDimensions::new(1, 1),
            ),
            (
                FROG.into(),
                PathBuf::from("res/images/spriteSheets/frog.png"),
                SpriteSheetDimensions::new(4, 1),
            ),
        ],
    };
    let scene = Scene {
        render_scene_descriptor: RenderSceneDescriptor {
            index_format: Index::index_format(),
            use_textures: true,
            vertex_buffer_layout: Vertex::describe_buffer_layout(),
        },
        z_index: 0,
        shader_descriptor,
        name: main_scene.into(),
        render_scene: main_scene.into(),
        target_window: MAIN_WINDOW.into(),
        entities: vec![
            Box::new(Player {
                name: protaginist_name.into(),
                size: PhysicalSize::new(64, 128),
                position: Vector::new(0.0, 0.0, 0.0),
                velocity: VelocityController::new(3.0),
                animation: Animation::new(
                    player_sprite_sheet.into(),
                    vec![
                        (Duration::from_millis(240), SpritePosition::new(0, 0)),
                        (Duration::from_millis(240), SpritePosition::new(1, 0)),
                        (Duration::from_millis(240), SpritePosition::new(2, 0)),
                        (Duration::from_millis(240), SpritePosition::new(3, 0)),
                    ],
                ),
            }),
            Box::new(Background {
                name: background.into(),
                size: PhysicalSize::new(1280, 800),
                sprite_sheet: background.into(),
            }),
            Box::new(Enemy {
                name: FROG.into(),
                size: PhysicalSize::new(64, 64),
                position: Vector::new(100.0, 100.0, 0.0),
                animation: Animation::new(
                    FROG.into(),
                    vec![
                        (Duration::from_millis(240), SpritePosition::new(0, 0)),
                        (Duration::from_millis(240), SpritePosition::new(1, 0)),
                        (Duration::from_millis(240), SpritePosition::new(2, 0)),
                        (Duration::from_millis(240), SpritePosition::new(3, 0)),
                    ],
                ),
            }),
        ],
    };
    let mut app = ManagerApplication::new(Game::new(ressources, vec![scene], target_fps));
    app.run();
}
