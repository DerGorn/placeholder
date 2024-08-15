use env_logger::Env;
use log::debug;
use placeholder::app::{ManagerApplication, WindowDescriptor};
use placeholder::graphics::{RenderSceneDescriptor, ShaderDescriptor, Visibility};
// use rodio::{Decoder, OutputStream, Sink, Source};
use std::fmt::{Debug, Display};
// use std::fs::File;
// use std::io::BufReader;
use std::path::PathBuf;
use threed::Vector;
use ui::{Alignment, Button, ButtonStyle, FlexButtonLine, FlexDirection, FlexOrigin, FontSize};
use winit::keyboard::KeyCode;
use winit::{dpi::PhysicalSize, window::WindowAttributes};

use placeholder::graphics::{Index as I, Vertex as V};

use placeholder::game_engine::{
    CameraDescriptor, EntityName, EntityType, Game, RessourceDescriptor, Scene, SceneName,
    SpriteSheetDimensions, State,
};

mod static_camera;
use static_camera::static_camera;

mod animation;

mod background;

mod transition;

mod enemy;

mod player;

mod vertex;
use vertex::{SimpleVertex, UiVertex, Vertex};

mod ui;

mod color;

mod game_state;
use game_state::{BattleActionManager, GameState, Skill};

mod event;
use event::Event;

mod battle_manager;
use crate::battle_manager::BATTLE_MANAGER;
use crate::event::{BattleEvent, EntityEvent};
use crate::game_state::{BattleAction, TargetGroup, UIState};

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
#[derive(PartialEq, Clone)]
enum CharacterAlignment {
    Friendly,
    Enemy,
}
#[derive(Clone)]
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
trait KIBehavior {
    fn ki(
        &mut self,
        character_index: usize,
        character: &Character,
        skills: &[Box<dyn Skill>],
        action_manager: &mut BattleActionManager,
        characters: &[&SkilledCharacter],
        current_time: f32,
    );
}
struct NoKI;
impl KIBehavior for NoKI {
    fn ki(
        &mut self,
        _character_index: usize,
        _character: &Character,
        _skills: &[Box<dyn Skill>],
        _action_manager: &mut BattleActionManager,
        _characters: &[&SkilledCharacter],
        _current_time: f32,
    ) {
        // Do nothing
    }
}
struct SimpleKI;
impl KIBehavior for SimpleKI {
    fn ki(
        &mut self,
        character_index: usize,
        character: &Character,
        skills: &[Box<dyn Skill>],
        action_manager: &mut BattleActionManager,
        characters: &[&SkilledCharacter],
        current_time: f32,
    ) {
        let target_index = characters
            .iter()
            .position(|c| c.character.alignment == CharacterAlignment::Friendly)
            .unwrap();
        let target = &characters[target_index];
        let skill_index = 0;
        let skill = skills.get(skill_index).unwrap();
        let action = BattleAction::new(
            skill.get_time(Some(&target.character), &character, current_time),
            character_index,
            skill_index,
            target_index,
        );
        action_manager.queue_action(action);
    }
}
struct SkilledCharacter {
    character: Character,

    skills: Vec<Box<dyn Skill>>,
    ki: Box<dyn KIBehavior>,
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

struct KeyBindings {
    accept: Vec<KeyCode>,
    cancel: Vec<KeyCode>,
    check: Vec<KeyCode>,
}
impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            accept: vec![KeyCode::Enter, KeyCode::Space],
            cancel: vec![KeyCode::KeyX],
            check: vec![KeyCode::KeyC],
        }
    }
}

struct GameLogic {
    pending_battle: Option<(EnemyType, EntityName, SceneName)>,
    game_state: GameState,
    key_bindings: KeyBindings,
}
impl GameLogic {
    fn new() -> Self {
        Self {
            pending_battle: None,
            game_state: GameState::default(),
            key_bindings: KeyBindings::default(),
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
                            // vec![
                            //     Event::RequestDeleteScene(MAIN_MENU_SCENE.into()),
                            //     Event::RequestNewScenes(vec![Scene {
                            //         name: BATTLE_SCENE.into(),
                            //         render_scene: BATTLE_SCENE.into(),
                            //         target_window: MAIN_WINDOW.into(),
                            //         z_index: 0,
                            //         shader_descriptor: SHADER_UI_TEXTURE,
                            //         entities: vec![],
                            //     }]),
                            // ]
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
        let (battle_state, ui_state) = match &mut self.game_state {
            GameState::Battle(battle_state, ui_state) => (battle_state, ui_state),
            _ => return vec![],
        };
        match event {
            Event::NewScene(scene) if scene.as_str() == BATTLE_SCENE => {
                for source_index in 0..battle_state.characters.len() {
                    let (left, right) = battle_state.characters.split_at_mut(source_index);
                    let (character, right) = right.split_first_mut().unwrap();
                    let characters = left.iter().chain(right.iter()).collect::<Vec<_>>();
                    if character.character.alignment != CharacterAlignment::Enemy {
                        continue;
                    }
                    character.ki.ki(
                        source_index,
                        &character.character,
                        &character.skills,
                        &mut battle_state.actions,
                        &characters,
                        battle_state.current_time,
                    );
                }
                return vec![Event::EntityEvent(
                    BATTLE_MANAGER.into(),
                    EntityEvent::BattleHighlightValidSkillTargets(
                        battle_state
                            .characters
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| !battle_state.actions.contains_character(*i))
                            .map(|(_, c)| c.character.name.into())
                            .collect(),
                    ),
                )];
            }
            Event::BattleEvent(BattleEvent::NextAction) => {
                let action = battle_state.actions.pop();
                battle_state.current_time = action.time();
                action.act_out(&mut battle_state.characters);
                let characters = battle_state
                    .characters
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| i == &action.character_index() || i == &action.target_index())
                    .map(|(_, c)| c.character.clone())
                    .collect::<Vec<_>>();
                return vec![Event::EntityEvent(
                    BATTLE_MANAGER.into(),
                    EntityEvent::AnimateAction(action, characters),
                )];
            }
            Event::BattleEvent(BattleEvent::ActionConsequences(action)) => {
                let dead_characters = battle_state
                    .characters
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.character.health <= 0)
                    .collect::<Vec<_>>();
                for (dead_character, _) in &dead_characters {
                    battle_state.actions.remove_character(*dead_character)
                }
                let mut events = dead_characters
                    .iter()
                    .map(|(_, dead_character)| {
                        Event::EntityEvent(
                            BATTLE_MANAGER.into(),
                            EntityEvent::CharacterDeath(dead_character.character.name.into()),
                        )
                    })
                    .collect::<Vec<_>>();

                let character = battle_state.characters[action.character_index()]
                    .character
                    .name;
                battle_state.characters.retain(|c| c.character.health > 0);
                if battle_state
                    .characters
                    .iter()
                    .all(|c| c.character.alignment == CharacterAlignment::Friendly)
                {
                    todo!("Player Wins");
                } else if battle_state
                    .characters
                    .iter()
                    .all(|c| c.character.alignment == CharacterAlignment::Enemy)
                {
                    todo!("Player Loses");
                } else if let Some(character_index) = battle_state
                    .characters
                    .iter()
                    .position(|c| c.character.name == character)
                {
                    let (left, right) = battle_state.characters.split_at_mut(character_index);
                    let (character, right) = right.split_first_mut().unwrap();
                    let characters = left.iter().chain(right.iter()).collect::<Vec<_>>();
                    match character.character.alignment {
                        CharacterAlignment::Enemy => {
                            character.ki.ki(
                                character_index,
                                &character.character,
                                &mut character.skills,
                                &mut battle_state.actions,
                                &characters,
                                battle_state.current_time,
                            );
                        }
                        CharacterAlignment::Friendly => {
                            events.push(Event::EntityEvent(
                                BATTLE_MANAGER.into(),
                                EntityEvent::BattleHighlightValidSkillTargets(
                                    battle_state
                                        .characters
                                        .iter()
                                        .enumerate()
                                        .filter(|(i, _)| {
                                            !battle_state.actions.contains_character(*i)
                                        })
                                        .map(|(_, c)| c.character.name.into())
                                        .collect(),
                                ),
                            ));
                            return events;
                        }
                    };
                }
                events.push(Event::BattleEvent(BattleEvent::NextAction));
                return events;
            }
            Event::ButtonPressed(button, key_code) => {
                match (key_code, button.as_str(), &ui_state) {
                    (accept, END_GAME_BUTTON, _) if self.key_bindings.accept.contains(&accept) => {
                        return vec![Event::EndGame];
                    }
                    (accept, character, UIState::CharacterSelection)
                        if battle_state
                            .characters
                            .iter()
                            .position(|c| c.character.name == character)
                            .map_or(false, |i| !battle_state.actions.contains_character(i))
                            && self.key_bindings.accept.contains(&accept) =>
                    {
                        let character_index = battle_state
                            .characters
                            .iter()
                            .position(|c| c.character.name == character)
                            .unwrap();
                        let character = &battle_state.characters[character_index];
                        let skills = character
                            .skills
                            .iter()
                            .enumerate()
                            .map(|(i, s)| {
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
                            })
                            .collect();
                        *ui_state = UIState::ActionSelection(character_index);
                        return vec![
                            Event::RequestSuspendScene(BATTLE_SCENE.into()),
                            Event::RequestActivateSuspendedScene(
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                            ),
                            Event::RequestAddEntities(
                                vec![Box::new(FlexButtonLine::new(
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
                                ))],
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                            ),
                            Event::RequestSetVisibilityScene(
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                                Visibility::Visible,
                            ),
                        ];
                    }
                    (accept, skill, UIState::ActionSelection(character_index))
                        if self.key_bindings.accept.contains(&accept) =>
                    {
                        let character = &battle_state.characters[*character_index];
                        let skill_index = character
                            .skills
                            .iter()
                            .position(|s| s.name() == skill.into())
                            .unwrap();
                        let skill = &character.skills[skill_index];
                        let targets =
                            skill
                                .target_groups()
                                .into_iter()
                                .fold(vec![], |mut targets, group| {
                                    let valid_targets =
                                        battle_state.characters.iter().filter(|c| match group {
                                            TargetGroup::Friends => {
                                                c.character.alignment
                                                    == character.character.alignment
                                                    && c.character.name != character.character.name
                                            }
                                            TargetGroup::Enemies => {
                                                c.character.alignment
                                                    != character.character.alignment
                                                    && c.character.name != character.character.name
                                            }
                                            TargetGroup::Ownself => {
                                                c.character.name == character.character.name
                                            }
                                        });
                                    targets.extend(valid_targets);
                                    targets
                                });
                        let targets = targets
                            .iter()
                            .map(|c| c.character.name.into())
                            .collect::<Vec<_>>();
                        *ui_state = UIState::TargetSelection(*character_index, skill_index);
                        return vec![
                            Event::RequestSuspendScene(
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                            ),
                            Event::RequestActivateSuspendedScene(BATTLE_SCENE.into()),
                            Event::EntityEvent(
                                BATTLE_MANAGER.into(),
                                EntityEvent::BattleHighlightValidSkillTargets(targets),
                            ),
                        ];
                    }
                    (accept, target, UIState::TargetSelection(character_index, skill_index))
                        if self.key_bindings.accept.contains(&accept) =>
                    {
                        let source = &battle_state.characters[*character_index];
                        let skill = &source.skills[*skill_index];
                        let target_index = &battle_state
                            .characters
                            .iter()
                            .position(|c| c.character.name == target)
                            .unwrap();
                        let target = &battle_state.characters[*target_index];
                        let is_valid_target =
                            skill.target_groups().iter().any(|group| match group {
                                TargetGroup::Friends => {
                                    target.character.alignment == source.character.alignment
                                        && target.character.name != source.character.name
                                }
                                TargetGroup::Enemies => {
                                    target.character.alignment != source.character.alignment
                                        && target.character.name != source.character.name
                                }
                                TargetGroup::Ownself => {
                                    target.character.name == source.character.name
                                }
                            });
                        if !is_valid_target {
                            debug!(
                                "Invalid target: {} for {:?} of {}",
                                target.character.name,
                                skill.name(),
                                source.character.name
                            );
                            return vec![];
                        }
                        let action = BattleAction::new(
                            skill.get_time(
                                Some(&target.character),
                                &source.character,
                                battle_state.current_time,
                            ),
                            *character_index,
                            *skill_index,
                            *target_index,
                        );
                        battle_state.actions.queue_action(action);
                        let actionless_characters: Vec<EntityName> = battle_state
                            .characters
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| !battle_state.actions.contains_character(*i))
                            .map(|(_, c)| c.character.name.into())
                            .collect();

                        *ui_state = UIState::CharacterSelection;
                        let mut events = vec![
                            Event::RequestSetVisibilityScene(
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                                Visibility::Hidden,
                            ),
                            Event::RequestSuspendScene(
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                            ),
                            Event::RequestDeleteEntity(
                                "ActionSelectionLine".into(),
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                            ),
                        ];
                        events.append(&mut if actionless_characters.is_empty() {
                            vec![Event::BattleEvent(BattleEvent::NextAction)]
                        } else {
                            vec![Event::EntityEvent(
                                BATTLE_MANAGER.into(),
                                EntityEvent::BattleHighlightValidSkillTargets(
                                    actionless_characters,
                                ),
                            )]
                        });
                        return events;
                    }
                    (cancel, _, UIState::TargetSelection(character_index, _))
                        if self.key_bindings.cancel.contains(&cancel) =>
                    {
                        *ui_state = UIState::ActionSelection(*character_index);
                        return vec![
                            Event::EntityEvent(
                                BATTLE_MANAGER.into(),
                                EntityEvent::BattleHighlightValidSkillTargets(vec![]),
                            ),
                            Event::RequestRenderScene(BATTLE_SCENE.into()),
                            Event::RequestActivateSuspendedScene(
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                            ),
                            Event::RequestSuspendScene(BATTLE_SCENE.into()),
                        ];
                    }
                    (cancel, _, UIState::ActionSelection(_))
                        if self.key_bindings.cancel.contains(&cancel) =>
                    {
                        *ui_state = UIState::CharacterSelection;
                        return vec![
                            Event::RequestSuspendScene(
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                            ),
                            Event::RequestSetVisibilityScene(
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                                Visibility::Hidden,
                            ),
                            Event::RequestSetVisibilityScene(
                                BATTLE_SCENE.into(),
                                Visibility::Visible,
                            ),
                            Event::RequestActivateSuspendedScene(BATTLE_SCENE.into()),
                            Event::RequestDeleteEntity(
                                "ActionSelectionLine".into(),
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                            ),
                        ];
                    }
                    (cancel, BATTLE_DETAIL_OVERLAY, UIState::CharacterDetail(source_state))
                        if self.key_bindings.cancel.contains(&cancel) =>
                    {
                        let mut events = if matches!(**source_state, UIState::TargetSelection(_, _))
                        {
                            vec![Event::RequestSetVisibilityScene(
                                BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                                Visibility::Visible,
                            )]
                        } else {
                            vec![]
                        };
                        *ui_state = (**source_state).clone();
                        events.extend([
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
                        ]);
                        return events;
                    }
                    (check, name, source_state)
                        if self.key_bindings.check.contains(&check)
                            && match source_state {
                                UIState::CharacterSelection | UIState::TargetSelection(_, _) => {
                                    true
                                }
                                _ => false,
                            } =>
                    {
                        if let Some(character) = battle_state
                            .characters
                            .iter()
                            .find(|c| c.character.name == name)
                        {
                            let mut events =
                                if matches!(**source_state, UIState::TargetSelection(_, _)) {
                                    vec![
                                        Event::RequestSuspendScene(
                                            BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                                        ),
                                        Event::RequestSetVisibilityScene(
                                            BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                                            Visibility::Hidden,
                                        ),
                                    ]
                                } else {
                                    vec![]
                                };
                            *ui_state =
                                UIState::CharacterDetail(Box::new((**source_state).clone()));
                            events.extend([
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
                            ]);
                            return events;
                        }
                    }
                    _ => {}
                };
                if matches!(key_code, KeyCode::Enter | KeyCode::Space) {
                    // let name = entity.as_str();
                    // match name {
                    //     BATTLE_ATTACK_BUTTON => {
                    //         let remaining_actions = battle_state
                    //             .character_order
                    //             .iter()
                    //             .enumerate()
                    //             .filter(|(i, ci)| {
                    //                 ci == &&player_index
                    //                     && battle_state.actions.iter().find(|a| a.0 == *i).is_none()
                    //             })
                    //             .collect::<Vec<_>>();
                    //         let action = remaining_actions[0];
                    //         battle_state.actions.push((action.0, 0, enemy_index));
                    //         if remaining_actions.len() == 1 {
                    //             return vec![Event::BattleEvent(BattleEvent::FinishedPlanning)];
                    //         }
                    //     }
                    //     _ => {}
                    // }
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
