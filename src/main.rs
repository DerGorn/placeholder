use env_logger::Env;
use placeholder::app::{ManagerApplication, WindowDescriptor};
use placeholder::graphics::{RenderSceneDescriptor, ShaderDescriptor, UniformBufferName};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;
use threed::Vector;
use transition::{Transition, TransitionTypes};
use ui::{Alignment, FlexBox, FlexDirection, Image};
use winit::{dpi::PhysicalSize, window::WindowAttributes};

use placeholder::graphics::{Index as I, Vertex as V};

use placeholder::game_engine::{
    CameraDescriptor, EntityName, EntityType, ExternalEvent, Game, RessourceDescriptor, Scene,
    SceneName, SpritePosition, SpriteSheetDimensions, State, VelocityController,
};

mod static_camera;
use static_camera::static_camera;

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
use vertex::{SimpleVertex, UiVertex, Vertex};

mod ui;
use ui::Text;

mod color;
use color::Color;

type Index = u16;

#[derive(Debug, PartialEq)]
enum Type {
    Background,
    Player,
    Enemy,
    Menu,
}
impl EntityType for Type {}

#[derive(Debug, Clone)]
enum EnemyType {
    Frog,
}
#[derive(Debug)]
enum Event {
    RequestNewScenes(Vec<Scene<Self>>),
    NewScene(SceneName),
    UpdateUniformBuffer(UniformBufferName, Vec<u8>),
    InitiateBattle(EnemyType, EntityName, SceneName),
    AnimationEnded(EntityName),
    RequestSuspendScene(SceneName),
    RequestActivateSuspendedScene(SceneName),
    RequestDeleteScene(SceneName),
    RequestDeleteEntity(EntityName, SceneName),
}
impl ExternalEvent for Event {
    type EntityType = Type;
    fn is_request_suspend_scene<'a>(&'a self) -> Option<&'a SceneName> {
        match self {
            Event::RequestSuspendScene(scene) => Some(scene),
            _ => None,
        }
    }
    fn is_request_activate_suspended_scene<'a>(&'a self) -> Option<&'a SceneName> {
        match self {
            Event::RequestActivateSuspendedScene(scene) => Some(scene),
            _ => None,
        }
    }
    fn is_request_delete_scene<'a>(&'a self) -> Option<&'a SceneName> {
        match self {
            Event::RequestDeleteScene(scene) => Some(scene),
            _ => None,
        }
    }
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

    fn is_update_uniform_buffer<'a>(
        &'a self,
    ) -> Option<(&'a placeholder::graphics::UniformBufferName, &'a [u8])> {
        match self {
            Event::UpdateUniformBuffer(name, contents) => Some((name, contents)),
            _ => None,
        }
    }
    fn is_delete_entity<'a>(&'a self) -> Option<(&'a EntityName, &'a SceneName)> {
        match self {
            Event::RequestDeleteEntity(entity, scene) => Some((entity, scene)),
            _ => None,
        }
    }
}

const TRANSITION_NAME: &str = "BattleTransition";
struct PlayerState {
    health: u8,
    attack: u8,
}
impl PlayerState {
    fn new() -> Self {
        Self {
            health: 0,
            attack: 0,
        }
    }
}
struct GameState {
    player: PlayerState,
    pending_battle: Option<(EnemyType, EntityName, SceneName)>,
}
impl GameState {
    fn new() -> Self {
        Self {
            player: PlayerState::new(),
            pending_battle: None,
        }
    }
}
impl State<Event> for GameState {
    fn handle_event(&mut self, event: Event) -> Vec<Event> {
        match event {
            Event::InitiateBattle(enemy, entity, scene) => {
                if self.pending_battle.is_none() {
                    let shader_descriptor = ShaderDescriptor {
                        file: "res/shader/transition.wgsl",
                        vertex_shader: "vs_main",
                        fragment_shader: "fs_main",
                        uniforms: vec![UTIME],
                    };
                    self.pending_battle = Some((enemy, entity, scene.clone()));
                    return vec![
                        Event::RequestNewScenes(vec![Scene {
                            name: BATTLE_TRANSITION_SCENE.into(),
                            render_scene: BATTLE_TRANSITION_SCENE.into(),
                            target_window: MAIN_WINDOW.into(),
                            z_index: 1,
                            entities: vec![Box::new(Transition::new(
                                TransitionTypes::BattleTransition,
                                TRANSITION_NAME,
                                Duration::from_millis(750),
                            ))],
                            shader_descriptor,
                        }]),
                        Event::RequestSuspendScene(scene),
                    ];
                }
            }
            Event::AnimationEnded(animation_entity) => {
                if animation_entity == TRANSITION_NAME.into() {
                    let response = if let Some((enemy, entity, scene)) = &self.pending_battle {
                        println!("Starting Battle!");
                        vec![
                            Event::RequestDeleteEntity(entity.clone(), scene.clone()),
                            Event::RequestDeleteScene(BATTLE_TRANSITION_SCENE.into()),
                            Event::RequestActivateSuspendedScene(scene.clone()),
                        ]
                    } else {
                        vec![]
                    };
                    if response.len() > 0 {
                        self.pending_battle = None;
                    }
                    return response;
                }
            }
            _ => {}
        }
        vec![]
    }
}

const MAIN_WINDOW: &str = "MainWindow";
const MAIN_SCENE: &str = "MainScene";
const MAIN_MENU_SCENE: &str = "MainMenuScene";
const BATTLE_TRANSITION_SCENE: &str = "BattleTransitionScene";
const UTIME: &str = "Time";
const UUI_CAMERA: &str = "UICamera";
const FROG: &str = "Frog";
const FONT: &str = "Font";
fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let target_fps = 60;

    let cursor_path = "res/images/cursor/Cursor_Goth_Cursor.png";
    let main_window = WindowAttributes::default()
        .with_title("Wispers in the Void - Dark Dynasty")
        .with_inner_size(PhysicalSize::new(800, 600));
    let main_window_descriptor = WindowDescriptor::new(main_window).with_cursor(cursor_path);
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
    let title = "Title";
    let title_backdrop = "TitleBackdrop";
    let ressources = RessourceDescriptor {
        windows: vec![(MAIN_WINDOW.into(), main_window_descriptor)],
        uniforms: vec![
            (
                UTIME.into(),
                bytemuck::cast_slice(&[0.0_f32]).to_vec(),
                wgpu::ShaderStages::FRAGMENT,
            ),
            (
                UUI_CAMERA.into(),
                bytemuck::cast_slice(&static_camera(PhysicalSize::new(800.0, 600.0))).to_vec(),
                wgpu::ShaderStages::VERTEX,
            ),
        ],
        default_render_scene: (
            None,
            RenderSceneDescriptor {
                index_format: Index::index_format(),
                use_textures: true,

                vertex_buffer_layout: Vertex::describe_buffer_layout(),
            },
        ),
        render_scenes: vec![
            (
                MAIN_MENU_SCENE.into(),
                None,
                RenderSceneDescriptor {
                    index_format: Index::index_format(),
                    use_textures: true,
                    vertex_buffer_layout: UiVertex::describe_buffer_layout(),
                },
            ),
            (
                MAIN_SCENE.into(),
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
            (
                FONT.into(),
                PathBuf::from("res/fonts/font.png"),
                SpriteSheetDimensions::new(16, 16),
            ),
            (
                title.into(),
                PathBuf::from("res/images/spriteSheets/title.png"),
                SpriteSheetDimensions::new(1, 1),
            ),
            (
                title_backdrop.into(),
                PathBuf::from("res/images/spriteSheets/titleBackDrop.png"),
                SpriteSheetDimensions::new(1, 1),
            ),
        ],
    };
    // let main_scene = Scene {
    //     z_index: 0,
    //     shader_descriptor: ShaderDescriptor {
    //         file: "res/shader/texture_array.wgsl",
    //         vertex_shader: "vs_main",
    //         fragment_shader: "fs_main",
    //         uniforms: vec![],
    //     },
    //     name: MAIN_SCENE.into(),
    //     render_scene: MAIN_SCENE.into(),
    //     target_window: MAIN_WINDOW.into(),
    //     entities: vec![
    //         Box::new(Player {
    //             name: protaginist_name.into(),
    //             size: PhysicalSize::new(64, 128),
    //             position: Vector::new(0.0, 0.0, 0.0),
    //             velocity: VelocityController::new(3.0),
    //             animation: Animation::new(
    //                 player_sprite_sheet.into(),
    //                 vec![
    //                     (Duration::from_millis(240), SpritePosition::new(0, 0)),
    //                     (Duration::from_millis(240), SpritePosition::new(1, 0)),
    //                     (Duration::from_millis(240), SpritePosition::new(2, 0)),
    //                     (Duration::from_millis(240), SpritePosition::new(3, 0)),
    //                 ],
    //                 false,
    //             ),
    //         }),
    //         Box::new(Background {
    //             name: background.into(),
    //             size: PhysicalSize::new(1280, 800),
    //             sprite_sheet: background.into(),
    //         }),
    //         Box::new(Enemy {
    //             name: FROG.into(),
    //             size: PhysicalSize::new(64, 64),
    //             position: Vector::new(100.0, 100.0, 0.0),
    //             animation: Animation::new(
    //                 FROG.into(),
    //                 vec![
    //                     (Duration::from_millis(240), SpritePosition::new(0, 0)),
    //                     (Duration::from_millis(240), SpritePosition::new(1, 0)),
    //                     (Duration::from_millis(240), SpritePosition::new(2, 0)),
    //                     (Duration::from_millis(240), SpritePosition::new(3, 0)),
    //                 ],
    //                 false,
    //             ),
    //             enemy_type: EnemyType::Frog,
    //         }),
    //     ],
    // };

    // let main_menu_background_scene = Scene {
    //     z_index: 0,
    //     shader_descriptor: ShaderDescriptor {
    //         file: "res/shader/texture_array.wgsl",
    //         vertex_shader: "vs_main",
    //         fragment_shader: "fs_main",
    //         uniforms: vec![UUI_CAMERA],
    //     },
    //     name: MAIN_MENU_BACKGROUND_SCENE.into(),
    //     render_scene: MAIN_MENU_BACKGROUND_SCENE.into(),
    //     target_window: MAIN_WINDOW.into(),
    //     entities: vec![
    //         Box::new(Background {
    //             name: background.into(),
    //             size: PhysicalSize::new(800, 600),
    //             sprite_sheet: background.into(),
    //         }),
    //         Box::new(Image {
    //             name: title_backdrop.into(),
    //             dimensions: PhysicalSize::new(800, 200),
    //             position: Vector::new(0.0, 200.0, 0.0),
    //             image: (title_backdrop.into(), SpritePosition::new(0, 0)),
    //         }),
    //     ],
    // };
    let main_menu_text_scene = Scene {
        z_index: 1,
        shader_descriptor: ShaderDescriptor {
            file: "res/shader/ui_texture.wgsl",
            vertex_shader: "vs_main",
            fragment_shader: "fs_main",
            uniforms: vec![UUI_CAMERA],
        },
        name: MAIN_MENU_SCENE.into(),
        render_scene: MAIN_MENU_SCENE.into(),
        target_window: MAIN_WINDOW.into(),
        entities: vec![Box::new(FlexBox::new(
            FlexDirection::Y,
            Alignment::Center,
            Some((background.into(), SpritePosition::new(0, 0))),
            // None,
            120.0,
            false,
            PhysicalSize::new(800, 600),
            Vector::new(0.0, 0.0, 0.0),
            "MainMenuText".into(),
            vec![
                Box::new(Image::new(
                    title.into(),
                    PhysicalSize::new(800, 200),
                    Vector::new(0.0, 200.0, 0.0),
                    (title.into(), SpritePosition::new(0, 0)),
                )),
                Box::new(FlexBox::new(
                    FlexDirection::Y,
                    Alignment::Center,
                    None,
                    20.0,
                    true,
                    PhysicalSize::new(800, 400),
                    Vector::new(0.0, 200.0, 0.0),
                    "MainMenuButtons".into(),
                    vec![
                        Box::new(Text::new(
                            String::from("New Game"),
                            Color::new_rgba(0, 0, 0, 255),
                            "StartButton".into(),
                            PhysicalSize::new(800, 600),
                            Vector::scalar(0.0),
                            40,
                            true,
                        )),
                        Box::new(Text::new(
                            String::from("End Game"),
                            Color::new_rgba(0, 0, 0, 255),
                            "EndButton".into(),
                            PhysicalSize::new(800, 600),
                            Vector::scalar(0.0),
                            40,
                            true,
                        )),
                    ],
                )),
            ],
        ))],
    };
    // todo!("PROMOTE CAMERA TO ENTITY. And implement a static camera with screen size");
    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let sink = Sink::try_new(&stream_handle).unwrap();
    //
    // let file = BufReader::new(File::open("res/audio/Jungle.mp3").unwrap());
    // let source = Decoder::new(file).unwrap().amplify(0.1);
    // sink.append(source);

    let mut app = ManagerApplication::new(Game::new(
        ressources,
        vec![main_menu_text_scene],
        target_fps,
        GameState::new(),
    ));
    app.run();
}
