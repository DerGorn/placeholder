use placeholder::{create_name_struct, game_engine::{Scene, SpritePosition}};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{
    color::Color, ui::{
        Alignment, Button, ButtonStyle, FlexBox, FlexButtonLine, FlexButtonLineManager,
        FlexDirection, FlexOrigin, FontSize, Image,
    }, Character, CharacterAlignment, Event, SkilledCharacter, BATTLE_ACTION_SELECTION_OVERLAY_SCENE, BATTLE_DETAIL_OVERLAY_SCENE, BATTLE_SCENE, CHARACTER_DISPLAY_LINES, END_GAME_BUTTON, MAIN_MENU_SCENE, MAIN_WINDOW, RESOLUTION, SHADER_UI_TEXTURE, START_GAME_BUTTON
};

#[derive(Debug)]
pub enum UIState {
    CharacterSelection,
    CharacterDetail,
    /// Index into BattleState.characters to indicate whos skills are being selected
    ActionSelection(usize),
    TargetSelection,
}
pub struct BattleState {
    pub characters: Vec<SkilledCharacter>,
    /// Index into characters
    pub character_order: Vec<usize>,
    pub turn_counter: u8,
    /// (Index into character_order, Index into characters[character_order] skill list, Index into
    /// characters for target)
    pub actions: Vec<(usize, usize, usize)>,
}
impl BattleState {
    pub fn generate_character_order(&mut self) {
        let mut character_order = vec![];
        let mut tempos = self
            .characters
            .iter()
            .enumerate()
            .map(|(i, c)| (i as isize, 1.0 / c.character.speed as f32))
            .collect::<Vec<_>>();
        tempos.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        let last_char = tempos.first().unwrap().0;
        loop {
            let (next_char, next_time) = tempos.pop().unwrap();
            character_order.push(next_char as usize);
            if next_char == last_char {
                break;
            }
            tempos.push((
                next_char,
                next_time + 1.0 / self.characters[next_char as usize].character.speed as f32,
            ));
            tempos.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        }
        self.character_order = character_order;
    }
}
create_name_struct!(SkillName);
pub trait Skill {
    fn name(&self) -> SkillName;
    fn evaluate(&self, target: Option<&mut Character>, source: &mut Character);
}
pub struct AttackSkill {}
impl Skill for AttackSkill {
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

            skills: vec![Box::new(AttackSkill {})],
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
        let mut battle_state = BattleState {
            characters,
            character_order: vec![],
            turn_counter: 1,
            actions: vec![],
        };
        battle_state.generate_character_order();
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

                let enemies = battle_state
                    .characters
                    .iter()
                    .filter(|c| c.character.alignment == CharacterAlignment::Enemy)
                    .map(|c| (format!("{}", c.character.name), c.character.to_string()))
                    .map(|(name, content)| {
                        Box::new(Button::new(
                            content,
                            name.into(),
                            PhysicalSize::new(400, character_text_height as u16),
                            Vector::scalar(0.0),
                            FontSize::new(font_size),
                            false,
                            ButtonStyle::default(),
                            // ButtonStyle::Plain(Color::from_str("white"), Color::from_str("black")),
                        ))
                    })
                    .collect::<Vec<_>>();
                let friends = battle_state
                    .characters
                    .iter()
                    .filter(|c| c.character.alignment == CharacterAlignment::Friendly)
                    .map(|c| (format!("{}", c.character.name), c.character.to_string()))
                    .map(|(name, content)| {
                        Box::new(Button::new(
                            content,
                            name.into(),
                            PhysicalSize::new(400, character_text_height as u16),
                            Vector::scalar(0.0),
                            FontSize::new(font_size),
                            false,
                            ButtonStyle::default(),
                            // ButtonStyle::Plain(Color::from_str("white"), Color::from_str("black")),
                        ))
                    })
                    .collect::<Vec<_>>();
                let enemies = FlexButtonLine::new(
                    FlexDirection::X,
                    FlexOrigin::Start,
                    Alignment::Center,
                    None,
                    50.0,
                    true,
                    RESOLUTION,
                    Vector::new(0.0, 0.0, 0.0),
                    "EnemyButtons".into(),
                    false,
                    enemies,
                );
                let friends = FlexButtonLine::new(
                    FlexDirection::X,
                    FlexOrigin::Start,
                    Alignment::Center,
                    None,
                    50.0,
                    true,
                    RESOLUTION,
                    Vector::new(0.0, 0.0, 0.0),
                    "FriendButtons".into(),
                    true,
                    friends,
                );
                let characters = FlexButtonLineManager::new(
                    FlexDirection::Y,
                    FlexOrigin::Start,
                    Alignment::Center,
                    None,
                    RESOLUTION.height as f32 - 200.0 - 2.0 * character_text_height,
                    false,
                    PhysicalSize::new(RESOLUTION.width, RESOLUTION.height - 200),
                    Vector::new(0.0, 0.0, 0.0),
                    "CharacterButtons".into(),
                    true,
                    vec![Box::new(enemies), Box::new(friends)],
                );
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
