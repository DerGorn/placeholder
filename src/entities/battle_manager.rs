use std::fmt::Debug;

use placeholder::game_engine::{Entity, EntityName};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{
    character::CharacterAlignment,
    event::{BattleEvent, EntityEvent, Event},
    game_state::BattleState,
    ui::{
        Alignment, Button, ButtonStyle, FlexButtonLine, FlexButtonLineManager, FlexDirection,
        FlexOrigin, FontSize,
    },
    Type, RESOLUTION,
};

pub const BATTLE_MANAGER: &str = "Battle Manager";

pub struct BattleManager {
    gui: Box<FlexButtonLineManager>,
}
impl BattleManager {
    pub fn new(battle_state: &BattleState, font_size: u8, character_text_height: f32) -> Self {
        let enemies = battle_state
            .characters
            .iter()
            .filter(|c| c.character.alignment() == &CharacterAlignment::Enemy)
            .map(|c| (format!("{}", c.character.name()), c.character.to_string()))
            .map(|(name, content)| {
                Box::new(Button::new(
                    content,
                    name.into(),
                    PhysicalSize::new(400, character_text_height as u16),
                    Vector::scalar(0.0),
                    FontSize::new(font_size),
                    false,
                    ButtonStyle::default(),
                ))
            })
            .collect::<Vec<_>>();
        let friends = battle_state
            .characters
            .iter()
            .filter(|c| c.character.alignment() == &CharacterAlignment::Friendly)
            .map(|c| (format!("{}", c.character.name()), c.character.to_string()))
            .map(|(name, content)| {
                Box::new(Button::new(
                    content,
                    name.into(),
                    PhysicalSize::new(400, character_text_height as u16),
                    Vector::scalar(0.0),
                    FontSize::new(font_size),
                    false,
                    ButtonStyle::default(),
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
        Self {
            gui: Box::new(FlexButtonLineManager::new(
                FlexDirection::Y,
                FlexOrigin::Start,
                Alignment::Center,
                None,
                RESOLUTION.height as f32 - 200.0 - 2.0 * character_text_height,
                false,
                PhysicalSize::new(RESOLUTION.width, RESOLUTION.height - 200),
                Vector::new(0.0, 0.0, 0.0),
                BATTLE_MANAGER.into(),
                true,
                vec![Box::new(enemies), Box::new(friends)],
            )),
        }
    }
}
impl Debug for BattleManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BattleManager")
    }
}
impl Entity<Type, Event> for BattleManager {
    fn handle_event(
        &mut self,
        event: <Event as placeholder::game_engine::ExternalEvent>::EntityEvent,
    ) -> Vec<Event> {
        match event {
            EntityEvent::BattleHighlightValidSkillTargets(valid_targets) => {
                for character_line in &mut self.gui.children {
                    for character in &mut character_line.children {
                        if valid_targets.contains(character.name()) {
                            character.set_highlighted(true);
                        } else {
                            character.set_highlighted(false);
                        }
                    }
                }
            }
            EntityEvent::AnimateAction(characters) => {
                for character in characters {
                    let character_line_index = match character.alignment() {
                        CharacterAlignment::Friendly => 1,
                        CharacterAlignment::Enemy => 0,
                    };
                    let character_line = &mut self.gui.children[character_line_index];
                    let chatacter_gui = character_line
                        .children
                        .iter_mut()
                        .find(|c| c.name().as_str() == character.name())
                        .expect(&format!(
                            "Character {:?} not found in gui",
                            character.name()
                        ));
                    chatacter_gui.set_content(character.to_string());
                }
                return vec![Event::BattleEvent(BattleEvent::ActionConsequences)];
            }
            EntityEvent::CharacterDeath(character) => {
                self.gui.delete_child_entity(&character);
            }
        }
        vec![]
    }
    fn entity_type(&self) -> Type {
        Type::Controller
    }
    fn bounding_box(&self) -> placeholder::game_engine::BoundingBox {
        self.gui.bounding_box()
    }
    fn name(&self) -> &placeholder::game_engine::EntityName {
        self.gui.name()
    }
    fn sprite_sheets(&self) -> Vec<&placeholder::game_engine::SpriteSheetName> {
        self.gui.sprite_sheets()
    }
    fn render(
        &mut self,
        vertices: &mut placeholder::app::VertexBuffer,
        indices: &mut placeholder::app::IndexBuffer,
        sprite_sheet: Vec<Option<&placeholder::game_engine::SpriteSheet>>,
    ) {
        self.gui.render(vertices, indices, sprite_sheet);
    }
    fn update(
        &mut self,
        entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &std::time::Duration,
        scene: &placeholder::game_engine::SceneName,
    ) -> Vec<Event> {
        self.gui.update(entities, delta_t, scene)
    }
    fn handle_key_input(&mut self, input: &winit::event::KeyEvent) -> Vec<Event> {
        self.gui.handle_key_input(input)
    }
    fn delete_child_entity(&mut self, name: &EntityName) {
        self.gui.delete_child_entity(name)
    }
}
