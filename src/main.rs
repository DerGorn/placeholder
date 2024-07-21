use env_logger::Env;
use placeholder::app::{ManagerApplication, WindowDescriptor};
use placeholder::graphics::{
    RenderSceneDescriptor, ShaderDescriptor, UniformBufferName, Visibility,
};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::Chars;
use std::time::Duration;
use threed::Vector;
use transition::{Transition, TransitionTypes};
use ui::{
    Alignment, Button, ButtonStyle, FlexBox, FlexButtonLine, FlexDirection, FlexOrigin, FontSize,
    Image,
};
use winit::keyboard::KeyCode;
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

mod game_state;
use game_state::{BattleState, GameState, Skill};

mod event;
use event::Event;

use crate::event::BattleEvent;
use crate::game_state::UIState;

type Index = u16;

#[derive(Debug, PartialEq)]
pub enum Type {
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

const TRANSITION_NAME: &str = "BattleTransition";
const SHADER_UI_TEXTURE: ShaderDescriptor = ShaderDescriptor {
    file: "res/shader/ui_texture.wgsl",
    vertex_shader: "vs_main",
    fragment_shader: "fs_main",
    uniforms: &[UUI_CAMERA],
};
#[derive(PartialEq)]
enum CharacterAlignment {
    Friendly,
    Enemy,
}
struct Character {
    name: &'static str,
    alignment: CharacterAlignment,

    max_health: u16,
    health: u16,
    max_stamina: u16,
    stamina: u16,
    exhaustion_threshold: u16,
    exhaustion: u16,

    speed: u16,
    attack: u16,
}
struct SkilledCharacter {
    character: Character,

    skills: Vec<Box<dyn Skill>>,
}
impl SkilledCharacter {
    pub fn activate_skill(&mut self, skill_index: usize, target: Option<&mut SkilledCharacter>) {
        self.skills[skill_index].evaluate(target.map(|c| &mut c.character), &mut self.character);
    }
}
impl Debug for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {{ HP: {}/{}, ST: {}/{}, EX: {}  ||  SPD: {}, ATK: {} }}",
            self.name,
            self.health,
            self.max_health,
            self.stamina,
            self.max_stamina,
            self.exhaustion,
            self.speed,
            self.attack
        )
    }
}
const CHARACTER_DISPLAY_LINES: f32 = 4.0;
impl Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}
HP: {}/{}
ST: {}/{}
EX: {}",
            self.name,
            self.health,
            self.max_health,
            self.stamina,
            self.max_stamina,
            self.exhaustion,
        )
    }
}

struct GameLogic {
    pending_battle: Option<(EnemyType, EntityName, SceneName)>,
    game_state: GameState,
}
impl GameLogic {
    fn new() -> Self {
        Self {
            pending_battle: None,
            game_state: GameState::default(),
        }
    }

    fn main_menu_event(&mut self, event: Event) -> Vec<Event> {
        match event {
            Event::ButtonPressed(entity, key_code) => {
                if matches!(key_code, KeyCode::Enter | KeyCode::Space) {
                    match entity.as_str() {
                        END_GAME_BUTTON => vec![Event::EndGame],
                        START_GAME_BUTTON => {
                            todo!("Start game");
                            vec![
                                Event::RequestDeleteScene(MAIN_MENU_SCENE.into()),
                                Event::RequestNewScenes(vec![Scene {
                                    name: BATTLE_SCENE.into(),
                                    render_scene: BATTLE_SCENE.into(),
                                    target_window: MAIN_WINDOW.into(),
                                    z_index: 0,
                                    shader_descriptor: SHADER_UI_TEXTURE,
                                    entities: vec![],
                                }]),
                            ]
                        }
                        _ => vec![],
                    }
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    fn battle_event(&mut self, event: Event) -> Vec<Event> {
        match &mut self.game_state {
            GameState::Battle(battle_state, ui_state) => {
                let player_index = battle_state
                    .characters
                    .iter()
                    .position(|c| c.character.name == "Player")
                    .unwrap();
                let enemy_index = battle_state
                    .characters
                    .iter()
                    .position(|c| c.character.name == "Enemy")
                    .unwrap();
                match event {
                    Event::BattleEvent(BattleEvent::FinishedPlanning) => {
                        println!("Playing out Turn");
                        let mut actions = vec![];
                        for (order_index, character_index) in
                            battle_state.character_order.iter().enumerate()
                        {
                            match battle_state.actions.iter().find(|a| a.0 == order_index) {
                                Some((_, skill, target_index)) => {
                                    actions.push((character_index, skill, target_index))
                                }
                                None => actions.push((character_index, &0, &player_index)),
                            };
                        }
                        println!("Actions: {:?}", actions);
                        for action in actions {
                            let (source_index, skill_index, target_index) = action;
                            let (source, target) = if source_index == target_index {
                                let (_, right) =
                                    battle_state.characters.split_at_mut(*target_index);
                                let (left, _) = right.split_at_mut(1);
                                let source = &mut left[0];
                                (source, None)
                            } else {
                                let (left, target) =
                                    battle_state.characters.split_at_mut(*target_index);
                                let (target, right) = target.split_at_mut(1);
                                let target = &mut target[0];
                                if source_index < target_index {
                                    (&mut left[*source_index], Some(target))
                                } else {
                                    (&mut right[source_index - target_index - 1], Some(target))
                                }
                            };
                            source.activate_skill(*skill_index, target);
                        }
                        battle_state.characters.retain(|c| c.character.health > 0);
                        if battle_state
                            .characters
                            .iter()
                            .all(|c| c.character.alignment == CharacterAlignment::Friendly)
                        {
                            println!("Player Wins");
                        } else if battle_state
                            .characters
                            .iter()
                            .all(|c| c.character.alignment == CharacterAlignment::Enemy)
                        {
                            println!("Player Loses");
                        } else {
                            battle_state.turn_counter += 1;
                            battle_state.actions.clear();
                            battle_state.generate_character_order();
                            return vec![Event::ButtonPressed(
                                BATTLE_PRINT_STATE_BUTTON.into(),
                                KeyCode::Space,
                            )];
                        }
                        return vec![Event::EndGame];
                    }
                    Event::ButtonPressed(entity, key_code) => {
                        if matches!(key_code, KeyCode::Enter | KeyCode::Space) {
                            let name = entity.as_str();
                            match name {
                                END_GAME_BUTTON => return vec![Event::EndGame],
                                BATTLE_PRINT_STATE_BUTTON => {
                                    println!("TURN: {}", battle_state.turn_counter);
                                    println!("TurnOrder: {:?}", battle_state.character_order);
                                    println!("----------------------------------\n");
                                    for (i, p) in battle_state.characters.iter().enumerate().filter(
                                        |(_, c)| {
                                            c.character.alignment == CharacterAlignment::Friendly
                                        },
                                    ) {
                                        println!("{i}: {:?}", p.character);
                                    }
                                    println!("\n----------------------------------\n");
                                    for (i, e) in battle_state.characters.iter().enumerate().filter(
                                        |(_, c)| c.character.alignment == CharacterAlignment::Enemy,
                                    ) {
                                        println!("{i}: {:?}", e.character);
                                    }
                                }
                                BATTLE_ATTACK_BUTTON => {
                                    let remaining_actions = battle_state
                                        .character_order
                                        .iter()
                                        .enumerate()
                                        .filter(|(i, ci)| {
                                            ci == &&player_index
                                                && battle_state
                                                    .actions
                                                    .iter()
                                                    .find(|a| a.0 == *i)
                                                    .is_none()
                                        })
                                        .collect::<Vec<_>>();
                                    let action = remaining_actions[0];
                                    battle_state.actions.push((action.0, 0, enemy_index));
                                    if remaining_actions.len() == 1 {
                                        return vec![Event::BattleEvent(
                                            BattleEvent::FinishedPlanning,
                                        )];
                                    }
                                }
                                BATTLE_ATTACK_TWO_BUTTON => {
                                    let enemy_two_index = battle_state
                                        .characters
                                        .iter()
                                        .position(|c| c.character.name == "Enemy Two")
                                        .unwrap();
                                    let remaining_actions = battle_state
                                        .character_order
                                        .iter()
                                        .enumerate()
                                        .filter(|(i, ci)| {
                                            ci == &&player_index
                                                && battle_state
                                                    .actions
                                                    .iter()
                                                    .find(|a| a.0 == *i)
                                                    .is_none()
                                        })
                                        .collect::<Vec<_>>();
                                    let action = remaining_actions[0];
                                    battle_state.actions.push((action.0, 0, enemy_two_index));
                                    if remaining_actions.len() == 1 {
                                        return vec![Event::BattleEvent(
                                            BattleEvent::FinishedPlanning,
                                        )];
                                    }
                                }
                                 _ if battle_state
                                .characters
                                .iter()
                                .any(|c| c.character.name == name)=>
                            {
                                    let character = battle_state
                                        .characters
                                        .iter()
                                        .find(|c| c.character.name == name)
                                        .unwrap();
                                if !matches!(ui_state, UIState::CharacterSelection) {
                                    return vec![];
                                }
                                let skills = character.skills.iter().enumerate().map(|(i, s)| {
                                    let name = s.name();
                                    let name = name.as_str();
                                    Box::new(Button::new(
                                        name.to_string(),
                                        name.into(),
                                        PhysicalSize::new(400, 100),
                                        Vector::<f32>::y_axis() * 100.0 * i as f32,
                                        FontSize::new(32),
                                        true,
                                        ButtonStyle::default(),
                                    ))
                                }).collect();
                                *ui_state = UIState::ActionSelection;
                                return vec![
                                    Event::RequestSuspendScene(BATTLE_SCENE.into()),
                                    Event::RequestSetVisibilityScene(
                                        BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                                        Visibility::Visible,
                                    ),
                                    Event::RequestActivateSuspendedScene(BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into()),
                                    Event::RequestAddEntities(
                                        vec![Box::new(
                                            FlexButtonLine::new(
                                                FlexDirection::Y,
                                                FlexOrigin::Center,
                                                Alignment::Center,
                                                None,
                                                0.0,
                                                true,
                                                PhysicalSize::new(400, 100),
                                                Vector::scalar(0.0),
                                                "ActionSelectionLine".into(),
                                                true,
                                                skills,
                                            ),
                                        )],
                                        BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                                    ),
                                ];
                            }

                                _ => {}
                            }
                        } else if matches!(key_code, KeyCode::KeyC) {
                            let name = entity.as_str();
                            if let Some(character) = battle_state
                                .characters
                                .iter()
                                .find(|c| c.character.name == name)
                            {
                                if !matches!(ui_state, UIState::CharacterSelection) {
                                    return vec![];
                                }
                                *ui_state = UIState::CharacterDetail;
                                return vec![
                                    Event::RequestSuspendScene(BATTLE_SCENE.into()),
                                    Event::RequestSetVisibilityScene(
                                        BATTLE_SCENE.into(),
                                        Visibility::Hidden,
                                    ),
                                    Event::RequestSetVisibilityScene(
                                        BATTLE_DETAIL_OVERLAY_SCENE.into(),
                                        Visibility::Visible,
                                    ),
                                    Event::RequestActivateSuspendedScene(
                                        BATTLE_DETAIL_OVERLAY_SCENE.into(),
                                    ),
                                    Event::RequestAddEntities(
                                        vec![Box::new(FlexButtonLine::new(
                                            FlexDirection::Y,
                                            FlexOrigin::Center,
                                            Alignment::Center,
                                            None,
                                            0.0,
                                            true,
                                            PhysicalSize::new(100, 100),
                                            Vector::scalar(0.0),
                                            "CharacterDetailLine".into(),
                                            true,
                                            vec![Box::new(Button::new(
                                                format!("{:#?}", character.character),
                                                BATTLE_DETAIL_OVERLAY.into(),
                                                RESOLUTION,
                                                Vector::scalar(0.0),
                                                FontSize::new(32),
                                                false,
                                                ButtonStyle::default(),
                                            ))],
                                        ))],
                                        BATTLE_DETAIL_OVERLAY_SCENE.into(),
                                    ),
                                ];
                            }
                        } else if matches!(key_code, KeyCode::KeyX) {
                            if entity.as_str() == BATTLE_DETAIL_OVERLAY {
                                if !matches!(ui_state, UIState::CharacterDetail) {
                                    return vec![];
                                }
                                *ui_state = UIState::CharacterSelection;
                                return vec![
                                    Event::RequestSuspendScene(BATTLE_DETAIL_OVERLAY_SCENE.into()),
                                    Event::RequestSetVisibilityScene(
                                        BATTLE_DETAIL_OVERLAY_SCENE.into(),
                                        Visibility::Hidden,
                                    ),
                                    Event::RequestSetVisibilityScene(
                                        BATTLE_SCENE.into(),
                                        Visibility::Visible,
                                    ),
                                    Event::RequestActivateSuspendedScene(BATTLE_SCENE.into()),
                                    Event::RequestDeleteEntity(
                                        "CharacterDetailLine".into(),
                                        BATTLE_DETAIL_OVERLAY_SCENE.into(),
                                    ),
                                ];
                            }
                        } 
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        vec![]
    }
}
const BATTLE_PRINT_STATE_BUTTON: &str = "BattlePrintState";
const BATTLE_ATTACK_BUTTON: &str = "BattleAttack";
const BATTLE_ATTACK_TWO_BUTTON: &str = "BattleAttackTwo";
const BATTLE_DETAIL_OVERLAY_SCENE: &str = "BattleDetailOverlayScene";
const BATTLE_DETAIL_OVERLAY: &str = "BattleDetailOverlay";
const BATTLE_ACTION_SELECTION_OVERLAY_SCENE: &str = "BattleActionSelectionOverlayScene";
const BATTLE_ACTION_SELECTION_OVERLAY: &str = "BattleActionSelectionOverlay";
impl State<Event> for GameLogic {
    fn start_scenes(&self) -> Vec<Scene<Event>> {
        self.game_state.get_start_scenes()
    }
    fn handle_event(&mut self, event: Event) -> Vec<Event> {
        match self.game_state {
            GameState::MainMenu => self.main_menu_event(event),
            GameState::Battle(_, _) => self.battle_event(event),
        }
        // match event {
        //     Event::InitiateBattle(enemy, entity, scene) => {
        //         if self.pending_battle.is_none() {
        //             let shader_descriptor = ShaderDescriptor {
        //                 file: "res/shader/transition.wgsl",
        //                 vertex_shader: "vs_main",
        //                 fragment_shader: "fs_main",
        //                 uniforms: vec![UTIME],
        //             };
        //             self.pending_battle = Some((enemy, entity, scene.clone()));
        //             return vec![
        //                 Event::RequestNewScenes(vec![Scene {
        //                     name: BATTLE_TRANSITION_SCENE.into(),
        //                     render_scene: BATTLE_TRANSITION_SCENE.into(),
        //                     target_window: MAIN_WINDOW.into(),
        //                     z_index: 1,
        //                     entities: vec![Box::new(Transition::new(
        //                         TransitionTypes::BattleTransition,
        //                         TRANSITION_NAME,
        //                         Duration::from_millis(750),
        //                     ))],
        //                     shader_descriptor,
        //                 }]),
        //                 Event::RequestSuspendScene(scene),
        //             ];
        //         }
        //     }
        //     Event::AnimationEnded(animation_entity) => {
        //         if animation_entity == TRANSITION_NAME.into() {
        //             let response = if let Some((enemy, entity, scene)) = &self.pending_battle {
        //                 println!("Starting Battle!");
        //                 vec![
        //                     Event::RequestDeleteEntity(entity.clone(), scene.clone()),
        //                     Event::RequestDeleteScene(BATTLE_TRANSITION_SCENE.into()),
        //                     Event::RequestActivateSuspendedScene(scene.clone()),
        //                 ]
        //             } else {
        //                 vec![]
        //             };
        //             if response.len() > 0 {
        //                 self.pending_battle = None;
        //             }
        //             return response;
        //         }
        //     }
        //     _ => {}
        // }
        // vec![]
    }
}

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
