use std::fmt::Debug;

use placeholder::game_engine::{Entity, EntityName};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{
    character::{CharacterAlignment, CHARACTER_TEXT_HEIGHT},
    event::{BattleEvent, EntityEvent, Event},
    game_state::BattleState,
    ui::{Alignment, FlexCharacterGuiLine, FlexCharacterGuiLineManager, FlexDirection, FlexOrigin},
    Type, RESOLUTION,
};

pub const BATTLE_MANAGER: &str = "Battle Manager";

pub struct BattleManager {
    gui: Box<FlexCharacterGuiLineManager>,
    pending_attack_animations: Vec<EntityName>,
}
impl BattleManager {
    pub fn new(battle_state: &BattleState) -> Self {
        let enemies = battle_state
            .characters
            .iter()
            .filter(|c| c.character.alignment() == &CharacterAlignment::Enemy)
            .map(|c| c.gui.create_gui(c))
            .collect::<Vec<_>>();
        let friends = battle_state
            .characters
            .iter()
            .filter(|c| c.character.alignment() == &CharacterAlignment::Friendly)
            .map(|c| c.gui.create_gui(c))
            .collect::<Vec<_>>();
        let enemies = FlexCharacterGuiLine::new(
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

        let friends = FlexCharacterGuiLine::new(
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

        let padding = 40;
        Self {
            gui: Box::new(FlexCharacterGuiLineManager::new(
                FlexDirection::Y,
                FlexOrigin::Start,
                Alignment::Center,
                None,
                RESOLUTION.height as f32 / 3.0 - padding as f32,
                false,
                PhysicalSize::new(RESOLUTION.width, RESOLUTION.height - padding),
                Vector::new(0.0, 0.0, 0.0),
                BATTLE_MANAGER.into(),
                true,
                vec![Box::new(enemies), Box::new(friends)],
            )),
            pending_attack_animations: vec![],
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
                let mut first = true;
                let mut focus_target = None;
                for (line_index, character_line) in self.gui.children.iter_mut().enumerate() {
                    for (character_index, character) in
                        character_line.children.iter_mut().enumerate()
                    {
                        if valid_targets.contains(character.name()) {
                            character.set_highlighted(true);
                            if first {
                                focus_target = Some((line_index, character_index));
                                first = false;
                            }
                        } else {
                            character.set_highlighted(false);
                        }
                    }
                }
                if let Some((line_index, character_index)) = focus_target {
                    self.gui.focus_child(line_index);
                    use crate::ui::FlexItem;
                    if self.gui.children[line_index]
                        .children
                        .iter()
                        .find(|c| c.has_focus())
                        .map_or(true, |focused_child| {
                            !valid_targets.contains(focused_child.name())
                        })
                    {
                        self.gui.children[line_index].focus_child(character_index);
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
                    self.pending_attack_animations
                        .append(&mut chatacter_gui.set_content(&character));
                }
                return if self.pending_attack_animations.is_empty() {
                    vec![Event::BattleEvent(BattleEvent::ActionConsequences)]
                } else {
                    vec![]
                };
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
        let mut events = self.gui.update(entities, delta_t, scene);
        if self.pending_attack_animations.is_empty() {
            return events;
        }
        for event in &events {
            match event {
                Event::AnimationEnded(bar) => {
                    self.pending_attack_animations.retain(|b| b != bar);
                }
                _ => {}
            }
        }
        if self.pending_attack_animations.is_empty() {
            events.push(Event::BattleEvent(BattleEvent::ActionConsequences));
        }
        events
    }
    fn handle_key_input(&mut self, input: &winit::event::KeyEvent) -> Vec<Event> {
        self.gui.handle_key_input(input)
    }
    fn delete_child_entity(&mut self, name: &EntityName) {
        self.gui.delete_child_entity(name)
    }
}
