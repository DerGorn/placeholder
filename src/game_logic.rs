use crate::{
    character::{skills::TargetGroup, CharacterAlignment},
    entities::BATTLE_MANAGER,
    event::{BattleEvent, EntityEvent},
    game_state::UIState,
    ui::{Alignment, Button, ButtonStyle, FlexButtonLine, FlexDirection, FlexOrigin, FontSize},
    BATTLE_ACTION_SELECTION_OVERLAY_SCENE, BATTLE_DETAIL_OVERLAY, BATTLE_DETAIL_OVERLAY_SCENE,
    BATTLE_SCENE, END_GAME_BUTTON, RESOLUTION, START_GAME_BUTTON,
};
use log::debug;
use placeholder::{
    game_engine::{EntityName, Scene, SceneName, State},
    graphics::Visibility,
};
use threed::Vector;
use winit::{dpi::PhysicalSize, keyboard::KeyCode};

use crate::{battle_action::BattleAction, event::Event, game_state::GameState, EnemyType};

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

pub struct GameLogic {
    pending_battle: Option<(EnemyType, EntityName, SceneName)>,
    game_state: GameState,
    key_bindings: KeyBindings,
}
impl GameLogic {
    pub fn new() -> Self {
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
                    let (character, right) =
                        right.split_first_mut().expect("source_index out of bounds");
                    let characters = left.iter().chain(right.iter()).collect::<Vec<_>>();
                    if character.character.alignment() != &CharacterAlignment::Enemy {
                        continue;
                    }
                    character.ki(
                        source_index,
                        &characters,
                        &mut battle_state.actions,
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
                            .map(|(_, c)| c.character.name().into())
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
                    EntityEvent::AnimateAction(characters),
                )];
            }
            Event::BattleEvent(BattleEvent::ActionConsequences) => {
                let dead_characters = battle_state
                    .characters
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.character.is_dead())
                    .collect::<Vec<_>>();
                for (dead_character, _) in &dead_characters {
                    battle_state.actions.remove_character(*dead_character)
                }
                let mut events = dead_characters
                    .iter()
                    .map(|(_, dead_character)| {
                        Event::EntityEvent(
                            BATTLE_MANAGER.into(),
                            EntityEvent::CharacterDeath(dead_character.character.name().into()),
                        )
                    })
                    .collect::<Vec<_>>();

                battle_state.characters.retain(|c| !c.character.is_dead());
                if battle_state
                    .characters
                    .iter()
                    .all(|c| c.character.alignment() == &CharacterAlignment::Friendly)
                {
                    todo!("Player Wins");
                } else if battle_state
                    .characters
                    .iter()
                    .all(|c| c.character.alignment() == &CharacterAlignment::Enemy)
                {
                    todo!("Player Loses");
                }
                let free_characters: Vec<_> = battle_state
                    .characters
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !battle_state.actions.contains_character(*i))
                    .collect();

                let free_friendly_characters: Vec<_> = free_characters
                    .iter()
                    .filter(|(_, c)| c.character.alignment() == &CharacterAlignment::Friendly)
                    .map(|(_, c)| c.character.name().into())
                    .collect();

                let free_enemies: Vec<usize> = free_characters
                    .iter()
                    .filter(|(_, c)| c.character.alignment() == &CharacterAlignment::Enemy)
                    .map(|(i, _)| *i)
                    .collect();
                for enemy_index in free_enemies {
                    let (left, right) = battle_state.characters.split_at_mut(enemy_index);
                    let (character, right) = right
                        .split_first_mut()
                        .expect("character_index out of bounds");
                    let characters = left.iter().chain(right.iter()).collect::<Vec<_>>();
                    match character.character.alignment() {
                        CharacterAlignment::Enemy => {
                            character.ki(
                                enemy_index,
                                &characters,
                                &mut battle_state.actions,
                                battle_state.current_time,
                            );
                        }
                        CharacterAlignment::Friendly => {}
                    };
                }

                if !free_friendly_characters.is_empty() {
                    events.push(Event::EntityEvent(
                        BATTLE_MANAGER.into(),
                        EntityEvent::BattleHighlightValidSkillTargets(free_friendly_characters),
                    ));
                    return events;
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
                            .position(|c| c.character.name() == character)
                            .map_or(false, |i| !battle_state.actions.contains_character(i))
                            && self.key_bindings.accept.contains(&accept) =>
                    {
                        let character_index = battle_state
                            .characters
                            .iter()
                            .position(|c| c.character.name() == character)
                            .expect(format!("Character {} not found", character).as_str());
                        let character = &battle_state.characters[character_index];
                        let skills = character
                            .get_skills()
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
                            .get_skills()
                            .iter()
                            .position(|s| s.name() == skill.into())
                            .expect(format!("Skill {} not found", skill).as_str());
                        let skill = &character.get_skills()[skill_index];
                        let targets =
                            skill
                                .target_groups()
                                .into_iter()
                                .fold(vec![], |mut targets, group| {
                                    let valid_targets =
                                        battle_state.characters.iter().filter(|c| match group {
                                            TargetGroup::Friends => {
                                                c.character.alignment()
                                                    == character.character.alignment()
                                                    && c.character.name()
                                                        != character.character.name()
                                            }
                                            TargetGroup::Enemies => {
                                                c.character.alignment()
                                                    != character.character.alignment()
                                                    && c.character.name()
                                                        != character.character.name()
                                            }
                                            TargetGroup::Ownself => {
                                                c.character.name() == character.character.name()
                                            }
                                        });
                                    targets.extend(valid_targets);
                                    targets
                                });
                        let targets = targets
                            .iter()
                            .map(|c| c.character.name().into())
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
                        let skill = &source.get_skills()[*skill_index];
                        let target_index = &battle_state
                            .characters
                            .iter()
                            .position(|c| c.character.name() == target)
                            .expect(format!("Character {} not found", target).as_str());
                        let target = &battle_state.characters[*target_index];
                        let is_valid_target =
                            skill.target_groups().iter().any(|group| match group {
                                TargetGroup::Friends => {
                                    target.character.alignment() == source.character.alignment()
                                        && target.character.name() != source.character.name()
                                }
                                TargetGroup::Enemies => {
                                    target.character.alignment() != source.character.alignment()
                                        && target.character.name() != source.character.name()
                                }
                                TargetGroup::Ownself => {
                                    target.character.name() == source.character.name()
                                }
                            });
                        if !is_valid_target {
                            debug!(
                                "Invalid target: {} for {:?} of {}",
                                target.character.name(),
                                skill.name(),
                                source.character.name()
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
                            .map(|(_, c)| c.character.name().into())
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
                            .find(|c| c.character.name() == name)
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
