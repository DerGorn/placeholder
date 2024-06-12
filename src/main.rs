use env_logger::Env;
use placeholder::app::{ManagerApplication, WindowDescriptor};
use placeholder::graphics::ShaderDescriptor;
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

mod vertex;
use vertex::Vertex;

mod game;
use game::{
    BoundingBox, CameraDescriptor, Direction, Entity, EntityName, EntityType, Game, Index,
    RessourceDescriptor, Scene, SpritePosition, SpriteSheet, SpriteSheetDimensions,
    SpriteSheetName, VelocityController,
};

struct Animation<T> {
    sprite_sheet: SpriteSheetName,
    keyframes: Vec<(Duration, T)>,
    current_keyframe: usize,
    time_since_frame_start: Duration,
}
impl<T> Animation<T> {
    fn new(sprite_sheet: SpriteSheetName, keyframes: Vec<(Duration, T)>) -> Self {
        Self {
            sprite_sheet,
            keyframes,
            current_keyframe: 0,
            time_since_frame_start: Duration::from_millis(0),
        }
    }

    fn update(&mut self, delta_t: &Duration) {
        self.time_since_frame_start += *delta_t;
        if self.time_since_frame_start >= self.keyframes[self.current_keyframe].0 {
            self.current_keyframe = (self.current_keyframe + 1) % self.keyframes.len();
            self.time_since_frame_start = Duration::from_millis(0);
        }
    }

    fn keyframe(&self) -> &T {
        &self.keyframes[self.current_keyframe].1
    }
}

#[derive(Debug, PartialEq)]
enum Type {
    Background,
    Player,
    Enemy,
}
impl EntityType for Type {}

struct Background {
    name: EntityName,
    sprite_sheet: SpriteSheetName,
    size: PhysicalSize<u16>,
}
impl Debug for Background {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Background")
            .field("z", &self.z())
            .field("sprite", &self.sprite_sheet())
            .finish()
    }
}
impl Entity<Type> for Background {
    fn entity_type(&self) -> Type {
        Type::Background
    }
    fn update(&mut self, _entities: &Vec<&Box<dyn Entity<Type>>>, _delta_t: &Duration) {}
    fn sprite_sheet(&self) -> &SpriteSheetName {
        &self.sprite_sheet
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn position(&self) -> Vector<f32> {
        Vector::new(0.0, 0.0, 0.0)
    }
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: Vector::new(0.0, 0.0, 0.0),
            size: PhysicalSize::new(self.size.width as f32, self.size.height as f32),
        }
    }
    fn z(&self) -> f32 {
        -1000.0
    }
    fn handle_key_input(&mut self, _input: &KeyEvent) {}
    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        sprite_sheet: &SpriteSheet,
    ) {
        self.render_sprite(vertices, indices, sprite_sheet, &SpritePosition::new(0, 0))
    }
}

struct Transition {
    name: EntityName,
    animation: Animation<(Vec<Vertex>, Vec<Index>)>,
}
impl Debug for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transition")
            .field("name", &self.name)
            .finish()
    }
}
impl Entity<Type> for Transition {
    fn update(&mut self, _entities: &Vec<&Box<dyn Entity<Type>>>, delta_t: &Duration) {
        self.animation.update(delta_t);
    }
    fn sprite_sheet(&self) -> &SpriteSheetName {
        todo!()
    }
    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        _sprite_sheet: &SpriteSheet,
    ) {
        let (new_vertices, new_indices) = self.animation.keyframe();
        vertices.extend(new_vertices.iter());
        indices.extend(new_indices.iter());
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn entity_type(&self) -> Type {
        Type::Background
    }
    fn handle_key_input(&mut self, _input: &KeyEvent) {}
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: Vector::new(0.0, 0.0, 0.0),
            size: PhysicalSize::new(10000.0, 10000.0),
        }
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
impl Entity<Type> for Enemy {
    fn update(&mut self, entities: &Vec<&Box<dyn Entity<Type>>>, delta_t: &Duration) {
        self.animation.update(delta_t);
        let players = entities.iter().filter(|e| e.entity_type() == Type::Player);
        let own_bounding_box = self.bounding_box();
        for player in players {
            let bounding_box = player.bounding_box();
            if own_bounding_box.intersects(&bounding_box) {
                println!(
                    "Player {:?} collided with enemy {:?}",
                    player.name(),
                    self.name()
                );
            }
        }
    }
    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        sprite_sheet: &SpriteSheet,
    ) {
        self.render_sprite(vertices, indices, sprite_sheet, self.animation.keyframe())
    }
    fn sprite_sheet(&self) -> &SpriteSheetName {
        &self.animation.sprite_sheet
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
impl Entity<Type> for Player {
    fn entity_type(&self) -> Type {
        Type::Player
    }
    fn update(&mut self, entities: &Vec<&Box<dyn Entity<Type>>>, delta_t: &Duration) {
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

    fn sprite_sheet(&self) -> &SpriteSheetName {
        &self.animation.sprite_sheet
    }

    fn z(&self) -> f32 {
        self.position.z
    }

    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        sprite_sheet: &SpriteSheet,
    ) {
        self.render_sprite(vertices, indices, sprite_sheet, self.animation.keyframe());
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
    let frog_name = "Frog";
    let protaginist_name = "Protagonist";
    let main_window = "MainWindow";
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
    let ressources = RessourceDescriptor {
        windows: vec![(
            main_window.into(),
            main_window_descriptor,
            camera_descriptor,
        )],
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
                frog_name.into(),
                PathBuf::from("res/images/spriteSheets/frog.png"),
                SpriteSheetDimensions::new(4, 1),
            ),
        ],
    };
    let scene = Scene {
        shader_descriptor,
        render_scene: "MainScene".into(),
        target_window: main_window.into(),
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
                name: frog_name.into(),
                size: PhysicalSize::new(64, 64),
                position: Vector::new(100.0, 100.0, 0.0),
                animation: Animation::new(
                    frog_name.into(),
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
