use env_logger::Env;
use placeholder::app::{ManagerApplication, WindowDescriptor};
use placeholder::graphics::{RenderSceneDescriptor, ShaderDescriptor};
use std::fmt::Debug;
use std::path::PathBuf;
use std::time::Duration;
use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    window::WindowAttributes,
};

use placeholder::graphics::{Index as I, Vertex as V};

use placeholder::game_engine::{
    CameraDescriptor, EntityType, ExternalEvent, Game,
    RessourceDescriptor, Scene, SceneName, SpritePosition, SpriteSheetDimensions,
    VelocityController,
};

mod animation;
use animation::Animation;

mod background;
use background::Background;

mod transition;

mod enemy;
use enemy::Enemy;

mod player;
use player::Player;

mod vertex;
use vertex::{SimpleVertex, Vertex};

type Index = u16;

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

const MAIN_WINDOW: &str = "MainWindow";
const BATTLE_TRANSITION_SCENE: &str = "BattleTransitionScene";
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
        render_scenes: vec![
            (
                main_scene.into(),
                Some(camera_descriptor),
                RenderSceneDescriptor {
                    index_format: Index::index_format(),
                    use_textures: true,
                    vertex_buffer_layout: Vertex::describe_buffer_layout(),
                },
            ),
            (
                BATTLE_TRANSITION_SCENE.into(),
                None,
                RenderSceneDescriptor {
                    index_format: Index::index_format(),
                    vertex_buffer_layout: SimpleVertex::describe_buffer_layout(),
                    use_textures: false,
                },
            ),
        ],
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
