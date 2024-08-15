use env_logger::Env;
use placeholder::app::{ManagerApplication, WindowDescriptor};
use placeholder::graphics::{RenderSceneDescriptor, ShaderDescriptor};
// use rodio::{Decoder, OutputStream, Sink, Source};
use std::fmt::Debug;
// use std::fs::File;
// use std::io::BufReader;
use std::path::PathBuf;
use winit::{dpi::PhysicalSize, window::WindowAttributes};

use placeholder::graphics::{Index as I, Vertex as V};

use placeholder::game_engine::{
    static_camera, CameraDescriptor, EntityType, Game, RessourceDescriptor, SpriteSheetDimensions,
};

mod animation;

mod entities;

mod vertex;
use vertex::{SimpleVertex, UiVertex, Vertex};

mod ui;

mod color;

mod game_state;

mod battle_action;

mod event;
use event::Event;

mod game_logic;
use game_logic::GameLogic;

mod character;
use character::{Character, SkilledCharacter};

type Index = u16;

#[derive(Debug, PartialEq)]
pub enum Type {
    Background,
    Player,
    Enemy,
    Menu,
    Controller,
}
impl EntityType for Type {}

#[derive(Debug, Clone)]
enum EnemyType {
    Frog,
}

const TRANSITION_NAME: &str = "BattleTransition";
const SHADER_UI_TEXTURE: ShaderDescriptor = ShaderDescriptor {
    file: "res/shader/ui_texture.wgsl",
    vertex_shader: "vs_main",
    fragment_shader: "fs_main",
    uniforms: &[UUI_CAMERA],
};
const BATTLE_PRINT_STATE_BUTTON: &str = "BattlePrintState";
const BATTLE_ATTACK_BUTTON: &str = "BattleAttack";
const BATTLE_ATTACK_TWO_BUTTON: &str = "BattleAttackTwo";
const BATTLE_DETAIL_OVERLAY_SCENE: &str = "BattleDetailOverlayScene";
const BATTLE_DETAIL_OVERLAY: &str = "BattleDetailOverlay";
const BATTLE_ACTION_SELECTION_OVERLAY_SCENE: &str = "BattleActionSelectionOverlayScene";
const BATTLE_ACTION_SELECTION_OVERLAY: &str = "BattleActionSelectionOverlay";

const MAIN_WINDOW: &str = "MainWindow";

const MAIN_MENU_SCENE: &str = "MainMenuScene";
const BATTLE_SCENE: &str = "BattleScene";
const MAIN_SCENE: &str = "MainScene";
const BATTLE_TRANSITION_SCENE: &str = "BattleTransitionScene";

const UTIME: &str = "Time";
const UUI_CAMERA: &str = "UICamera";
const FROG: &str = "Frog";
const FONT: &str = "Font";
const END_GAME_BUTTON: &str = "EndGameButton";
const START_GAME_BUTTON: &str = "StartGameButton";
const RESOLUTION: PhysicalSize<u16> = PhysicalSize::new(1920, 1080);
const FLOAT_RESOULTION: PhysicalSize<f32> =
    PhysicalSize::new(RESOLUTION.width as f32, RESOLUTION.height as f32);
fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let target_fps = 60;

    let cursor_path = "res/images/cursor/Cursor_Goth_Cursor.png";
    let main_window = WindowAttributes::default()
        .with_title("Wispers in the Void - Dark Dynasty")
        .with_inner_size(RESOLUTION.clone());
    let main_window_descriptor = WindowDescriptor::new(main_window).with_cursor(cursor_path);
    let protaginist_name = "Protagonist";
    let player_sprite_sheet = "PlayerSpriteSheet";
    let background = "Background";
    let camera_descriptor = CameraDescriptor {
        view_size: FLOAT_RESOULTION.clone(),
        speed: 90.0,
        acceleration_steps: 30,
        target_entity: protaginist_name.into(),
        bound_entity: Some(background.into()),
        max_offset_position: 100.0,
    };
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
                bytemuck::cast_slice(&static_camera(FLOAT_RESOULTION.clone())).to_vec(),
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
                vec![
                    BATTLE_SCENE.into(),
                    BATTLE_DETAIL_OVERLAY_SCENE.into(),
                    BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                    MAIN_MENU_SCENE.into(),
                ],
                None,
                RenderSceneDescriptor {
                    index_format: Index::index_format(),
                    use_textures: true,
                    vertex_buffer_layout: UiVertex::describe_buffer_layout(),
                },
            ),
            (
                vec![MAIN_SCENE.into()],
                Some(camera_descriptor),
                RenderSceneDescriptor {
                    index_format: Index::index_format(),
                    use_textures: true,
                    vertex_buffer_layout: Vertex::describe_buffer_layout(),
                },
            ),
            (
                vec![BATTLE_TRANSITION_SCENE.into()],
                None,
                RenderSceneDescriptor {
                    index_format: Index::index_format(),
                    vertex_buffer_layout: SimpleVertex::describe_buffer_layout(),
                    use_textures: false,
                },
            ),
        ],
        image_directory: PathBuf::from("res/images/spriteSheets/"),
        sprite_sheets: vec![
            (
                player_sprite_sheet.into(),
                PathBuf::from("res/images/spriteSheets/ProtagonistP.png"),
                SpriteSheetDimensions::new(4, 1),
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

    // todo!("PROMOTE CAMERA TO ENTITY. And implement a static camera with screen size");
    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let sink = Sink::try_new(&stream_handle).unwrap();
    //
    // let file = BufReader::new(File::open("res/audio/Jungle.mp3").unwrap());
    // let source = Decoder::new(file).unwrap().amplify(0.1);
    // sink.append(source);

    let mut app = ManagerApplication::new(Game::new(ressources, target_fps, GameLogic::new()));
    app.run();
}
