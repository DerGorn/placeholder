use placeholder::{
    create_name_struct,
    game_engine::{Scene, SpritePosition},
};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{
    battle_manager::BattleManager,
    color::Color,
    ui::{
        Alignment, Button, ButtonStyle, FlexBox, FlexButtonLine, FlexButtonLineManager,
        FlexDirection, FlexOrigin, FontSize, Image,
    },
    Character, CharacterAlignment, Event, SkilledCharacter, BATTLE_ACTION_SELECTION_OVERLAY_SCENE,
    BATTLE_DETAIL_OVERLAY_SCENE, BATTLE_SCENE, CHARACTER_DISPLAY_LINES, END_GAME_BUTTON,
    MAIN_MENU_SCENE, MAIN_WINDOW, RESOLUTION, SHADER_UI_TEXTURE, START_GAME_BUTTON,
};

#[derive(Debug, Clone)]
pub enum UIState {
    CharacterSelection,
    /// SourceState
    CharacterDetail(Box<UIState>),
    /// Index into BattleState.characters to indicate whos skills are being selected
    ActionSelection(usize),
    /// Index into BattleState.characters to indicate whos skills are being selected; Index into
    /// characters skill list
    TargetSelection(usize, usize),
}
pub struct BattleAction {
    time: f32,
    character_index: usize,
    skill_index: usize,
    target_character_index: usize,
}
impl BattleAction {
    pub fn new(
        time: f32,
        character_index: usize,
        skill_index: usize,
        target_character_index: usize,
    ) -> Self {
        Self {
            time,
            character_index,
            skill_index,
            target_character_index,
        }
    }
}
pub struct BattleActionManager {
    actions: Vec<BattleAction>,
}
impl BattleActionManager {
    pub fn queue_action(&mut self, action: BattleAction) {
        self.actions.push(action);
        self.actions.sort_by(|a, b| a.time.partial_cmp(&b.time).expect("Encountered NaN time"));
    }

    pub fn get_actions(&self) -> &[BattleAction] {
        &self.actions
    }
    
    pub fn contains_character(&self, character_index: usize) -> bool {
        self.actions.iter().any(|a| a.character_index == character_index)
    }
}
pub struct BattleState {
    pub characters: Vec<SkilledCharacter>,
    pub current_time: f32,
    pub actions: BattleActionManager,
}
create_name_struct!(SkillName);
#[derive(Debug)]
pub enum TargetGroup {
    Ownself,
    Friends,
    Enemies,
}
pub trait Skill {
    fn name(&self) -> SkillName;
    fn evaluate(&self, target: Option<&mut Character>, source: &mut Character);
    /// Relative TargetGroups
    fn target_groups(&self) -> Vec<TargetGroup>;
    fn get_time(&self, target: Option<&Character>, source: &Character) -> f32;
}
pub struct AttackSkill {}
impl Skill for AttackSkill {
    fn target_groups(&self) -> Vec<TargetGroup> {
        vec![TargetGroup::Enemies]
    }
    fn name(&self) -> SkillName {
        "Attack".into()
    }
    fn evaluate(&self, target: Option<&mut Character>, source: &mut Character) {
        let attack = source.attack;
        if let Some(target) = target {
            target.health -= attack.min(target.health);
        } else {
            source.health -= attack.min(source.health);
        }
    }
    fn get_time(&self, _target: Option<&Character>, source: &Character) -> f32 {
        source.speed as f32
    }
}
pub struct HealSkill {}
impl Skill for HealSkill {
    fn target_groups(&self) -> Vec<TargetGroup> {
        vec![TargetGroup::Friends, TargetGroup::Ownself]
    }
    fn name(&self) -> SkillName {
        "Heal".into()
    }
    fn evaluate(&self, target: Option<&mut Character>, source: &mut Character) {
        let heal = 10;
        if let Some(target) = target {
            target.health += heal.min(target.max_health - target.health);
        } else {
            source.health += heal.min(source.max_health - source.health);
        }
    }
    fn get_time(&self, _target: Option<&Character>, source: &Character) -> f32 {
        source.speed as f32
    }
}

pub enum GameState {
    MainMenu,
    /// Battle(FRIENDS, ENEMIES)
    Battle(BattleState, UIState),
}
impl Default for GameState {
    fn default() -> Self {
        let player = SkilledCharacter {
            character: Character {
                name: "Player",
                alignment: CharacterAlignment::Friendly,

                max_health: 100,
                health: 100,
                max_stamina: 10,
                stamina: 10,
                exhaustion_threshold: 100,
                exhaustion: 0,

                attack: 11,
                speed: 10,
            },

            skills: vec![Box::new(AttackSkill {}), Box::new(HealSkill {})],
        };
        let player_two = SkilledCharacter {
            character: Character {
                name: "Player Two",
                alignment: CharacterAlignment::Friendly,

                max_health: 50,
                health: 50,
                max_stamina: 20,
                stamina: 15,
                exhaustion_threshold: 150,
                exhaustion: 0,

                attack: 8,
                speed: 20,
            },

            skills: vec![Box::new(AttackSkill {})],
        };
        let player_three = SkilledCharacter {
            character: Character {
                name: "Player Three",
                alignment: CharacterAlignment::Friendly,

                max_health: 300,
                health: 300,
                max_stamina: 2,
                stamina: 2,
                exhaustion_threshold: 300,
                exhaustion: 100,

                attack: 4,
                speed: 2,
            },

            skills: vec![Box::new(AttackSkill {})],
        };
        let enemy = SkilledCharacter {
            character: Character {
                name: "Enemy",
                alignment: CharacterAlignment::Enemy,

                max_health: 100,
                health: 80,
                max_stamina: 5,
                stamina: 5,
                exhaustion_threshold: 10,
                exhaustion: 0,

                attack: 10,
                speed: 5,
            },

            skills: vec![Box::new(AttackSkill {})],
        };
        let enemy_two = SkilledCharacter {
            character: Character {
                name: "Enemy Two",
                alignment: CharacterAlignment::Enemy,
                max_health: 20,
                health: 20,
                max_stamina: 0,
                stamina: 0,
                exhaustion_threshold: 0,
                exhaustion: 0,
                attack: 15,
                speed: 12,
            },
            skills: vec![Box::new(AttackSkill {})],
        };

        let characters = vec![player, player_two, player_three, enemy, enemy_two];
        let battle_state = BattleState {
            characters,
            current_time: 0.0,
            actions: BattleActionManager { actions: vec![] },
        };
        Self::Battle(battle_state, UIState::CharacterSelection)
    }
}
impl GameState {
    pub fn get_start_scenes(&self) -> Vec<Scene<Event>> {
        match self {
            GameState::MainMenu => vec![Scene {
                z_index: 1,
                shader_descriptor: SHADER_UI_TEXTURE,
                name: MAIN_MENU_SCENE.into(),
                render_scene: MAIN_MENU_SCENE.into(),
                target_window: MAIN_WINDOW.into(),
                entities: vec![Box::new(FlexBox::new(
                    FlexDirection::Y,
                    FlexOrigin::Start,
                    Alignment::Center,
                    Some(("title_background".into(), SpritePosition::new(0, 0))),
                    330.0,
                    false,
                    RESOLUTION.clone(),
                    Vector::new(0.0, 0.0, 0.0),
                    "MainMenuText".into(),
                    vec![
                        Box::new(Image::new(
                            "title".into(),
                            PhysicalSize::new(800, 200),
                            Vector::scalar(0.0),
                            ("title".into(), SpritePosition::new(0, 0)),
                            Some(Color::new_rgba(82, 5, 5, 255)),
                        )),
                        Box::new(FlexButtonLine::new(
                            FlexDirection::Y,
                            FlexOrigin::Start,
                            Alignment::Center,
                            None,
                            20.0,
                            true,
                            PhysicalSize::new(800, 600),
                            Vector::new(0.0, 0.0, 0.0),
                            "MainMenuButtons".into(),
                            true,
                            vec![
                                Box::new(Button::new(
                                    String::from("New Game"),
                                    START_GAME_BUTTON.into(),
                                    PhysicalSize::new(800, 600),
                                    Vector::scalar(0.0),
                                    FontSize::new(40),
                                    true,
                                    ButtonStyle::default(),
                                )),
                                Box::new(Button::new(
                                    String::from("End Game"),
                                    END_GAME_BUTTON.into(),
                                    PhysicalSize::new(800, 600),
                                    Vector::scalar(0.0),
                                    FontSize::new(40),
                                    true,
                                    ButtonStyle::default(),
                                )),
                            ],
                        )),
                    ],
                ))],
            }],
            GameState::Battle(battle_state, UIState::CharacterSelection) => {
                let font_size = 32;
                let character_text_height = (CHARACTER_DISPLAY_LINES + 0.25) * font_size as f32;

                let characters = BattleManager::new(battle_state, font_size, character_text_height);
                vec![
                    Scene {
                        name: BATTLE_DETAIL_OVERLAY_SCENE.into(),
                        shader_descriptor: SHADER_UI_TEXTURE,
                        render_scene: BATTLE_DETAIL_OVERLAY_SCENE.into(),
                        target_window: MAIN_WINDOW.into(),
                        entities: vec![],
                        z_index: 0,
                    },
                    Scene {
                        name: BATTLE_SCENE.into(),
                        render_scene: BATTLE_SCENE.into(),
                        target_window: MAIN_WINDOW.into(),
                        z_index: 0,
                        shader_descriptor: SHADER_UI_TEXTURE,
                        entities: vec![Box::new(characters)],
                    },
                    Scene {
                        name: BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                        shader_descriptor: SHADER_UI_TEXTURE,
                        render_scene: BATTLE_ACTION_SELECTION_OVERLAY_SCENE.into(),
                        target_window: MAIN_WINDOW.into(),
                        entities: vec![],
                        z_index: 1,
                    },
                ]
            }
            GameState::Battle(_, ui_state) => {
                todo!("Implement UI for battle state: {:?}", ui_state)
            }
        }
    }
}
