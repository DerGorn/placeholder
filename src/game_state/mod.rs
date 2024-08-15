use crate::battle_action::BattleActionManager;
use crate::character::skills::AttackSkill;
use crate::character::skills::HealSkill;
use crate::character::CharacterAlignment;
use crate::character::NoKI;
use crate::character::SimpleKI;
use crate::character::CHARACTER_DISPLAY_LINES;
use placeholder::game_engine::{Scene, SpritePosition};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{
    color::Color,
    entities::BattleManager,
    ui::{
        Alignment, Button, ButtonStyle, FlexBox, FlexButtonLine, FlexDirection, FlexOrigin,
        FontSize, Image,
    },
    Character, Event, SkilledCharacter, BATTLE_ACTION_SELECTION_OVERLAY_SCENE,
    BATTLE_DETAIL_OVERLAY_SCENE, BATTLE_SCENE, END_GAME_BUTTON, MAIN_MENU_SCENE, MAIN_WINDOW,
    RESOLUTION, SHADER_UI_TEXTURE, START_GAME_BUTTON,
};

mod ui_state;
pub use ui_state::UIState;

mod battle_state;
pub use battle_state::BattleState;

pub enum GameState {
    MainMenu,
    Battle(BattleState, UIState),
}
impl Default for GameState {
    fn default() -> Self {
        let player = SkilledCharacter::new(
            Character::new("Player", CharacterAlignment::Friendly, 100, 10, 100, 10, 11),
            vec![Box::new(AttackSkill {}), Box::new(HealSkill {})],
            Box::new(NoKI),
        );
        let player_two = SkilledCharacter::new(
            Character::new(
                "Player Two",
                CharacterAlignment::Friendly,
                50,
                20,
                150,
                20,
                8,
            ),
            vec![Box::new(AttackSkill {})],
            Box::new(NoKI),
        );
        let player_three = SkilledCharacter::new(
            Character::new(
                "Player Three",
                CharacterAlignment::Friendly,
                300,
                2,
                300,
                2,
                4,
            ),
            vec![Box::new(AttackSkill {})],
            Box::new(NoKI),
        );
        let enemy = SkilledCharacter::new(
            Character::new("Enemy", CharacterAlignment::Enemy, 100, 5, 10, 5, 10),
            vec![Box::new(AttackSkill {})],
            Box::new(SimpleKI),
        );
        let enemy_two = SkilledCharacter::new(
            Character::new("Enemy Two", CharacterAlignment::Enemy, 20, 0, 0, 12, 15),
            vec![Box::new(AttackSkill {})],
            Box::new(SimpleKI),
        );

        let characters = vec![player, player_two, player_three, enemy, enemy_two];
        let battle_state = BattleState {
            characters,
            current_time: 0.0,
            actions: BattleActionManager::new(),
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
