use ferride_core::game_engine::{Entity, EntityName};
use ferride_core::reexports::winit::PhysicalSize;
use threed::Vector;

use crate::ui::button_styles::ColorPair;
use crate::ui::FontSize;
use crate::{
    character::Character,
    event::Event,
    ui::{Button, ButtonStyle, FlexItem, FlexProgressBarLine, ProgressBar},
    Type,
};

use super::{BAR_LOW_COLOR, BAR_PADDING, BAR_SIZE, CHARACTER_FONT_SIZE, CHARACTER_PORTRAIT_SIZE, EXHAUSTION_BAR_COLOR, HEALTH_BAR_COLOR, STAMINA_BAR_COLOR};

#[derive(Debug)]
pub struct CharacterGui {
    button: Box<Button>,
    bars: FlexProgressBarLine,
}
impl CharacterGui {
    pub fn with_button_style_and_character(style: ButtonStyle, character: &Character) -> Self {
        Self::new(
            Box::new(Button::new(
                String::new(),
                character.name.into(),
                CHARACTER_PORTRAIT_SIZE,
                Vector::scalar(0.0),
                FontSize::new(CHARACTER_FONT_SIZE),
                false,
                style,
            )),
            vec![
                Box::new(ProgressBar::new(
                    "health".into(),
                    BAR_SIZE,
                    Vector::scalar(0.0),
                    character.health,
                    character.max_health,
                    ColorPair::new(HEALTH_BAR_COLOR, BAR_LOW_COLOR),
                    BAR_PADDING,
                )),
                Box::new(ProgressBar::new(
                    "stamina".into(),
                    BAR_SIZE,
                    Vector::scalar(0.0),
                    character.stamina,
                    character.max_stamina,
                    ColorPair::new(STAMINA_BAR_COLOR, BAR_LOW_COLOR),
                    BAR_PADDING,
                )),
                Box::new(ProgressBar::new(
                    "exhaustion".into(),
                    BAR_SIZE,
                    Vector::scalar(0.0),
                    character.exhaustion,
                    character.exhaustion_threshold,
                    ColorPair::new(EXHAUSTION_BAR_COLOR, BAR_LOW_COLOR),
                    BAR_PADDING,
                )),
            ],
        )
    }

    pub fn new(button: Box<Button>, bars: Vec<Box<ProgressBar>>) -> Self {
        let bbox = button.bounding_box();
        Self {
            bars: FlexProgressBarLine::new(
                crate::ui::FlexDirection::Y,
                crate::ui::FlexOrigin::End,
                crate::ui::Alignment::End,
                None,
                0.0,
                false,
                PhysicalSize::new(bbox.size.width as u16, bbox.size.height as u16),
                bbox.anchor,
                button.name().clone() + "bars",
                false,
                bars,
            ),
            button,
        }
    }

    pub fn set_highlighted(&mut self, highlighted: bool) {
        self.button.set_highlighted(highlighted);
    }

    pub fn set_content(&mut self, character: &Character) -> Vec<EntityName> {
        let mut pending_animations = vec![];
        for bar in &mut self.bars.children {
            if match bar.name().as_str() {
                "health" => bar.set_value(character.health),
                "stamina" => bar.set_value(character.stamina),
                "exhaustion" => bar.set_value(character.exhaustion),
                x => {
                    unimplemented!(
                        "no stat '{}' on character, but a bar for it exists on '{}'",
                        x,
                        character.name
                    )
                }
            } {
                pending_animations.push(bar.name().clone());
            }
        }
        pending_animations
    }
}
impl Entity<Type, Event> for CharacterGui {
    fn update(
        &mut self,
        entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &std::time::Duration,
        scene: &ferride_core::game_engine::SceneName,
    ) -> Vec<Event> {
        self.bars.update(entities, delta_t, scene)
    }

    fn render(
        &mut self,
        vertices: &mut ferride_core::app::VertexBuffer,
        indices: &mut ferride_core::app::IndexBuffer,
        sprite_sheets: Vec<Option<&ferride_core::game_engine::SpriteSheet>>,
    ) {
        let mut index = 0;
        let amount_sprites = self.button.sprite_sheets().len();
        if sprite_sheets.len() >= amount_sprites {
            self.button.render(
                vertices,
                indices,
                sprite_sheets.get(0..amount_sprites).unwrap().to_vec(),
            );
            index = amount_sprites;
        }
        self.bars.render(
            vertices,
            indices,
            sprite_sheets.get(index..).unwrap().to_vec(),
        );
    }

    fn sprite_sheets(&self) -> Vec<&ferride_core::game_engine::SpriteSheetName> {
        let mut sprites = self.button.sprite_sheets();
        sprites.append(&mut self.bars.sprite_sheets());
        sprites
    }

    fn handle_key_input(
        &mut self,
        input: &ferride_core::reexports::winit::event::KeyEvent,
    ) -> Vec<Event> {
        self.button.handle_key_input(input)
    }

    fn name(&self) -> &ferride_core::game_engine::EntityName {
        self.button.name()
    }

    fn bounding_box(&self) -> ferride_core::game_engine::BoundingBox {
        self.button.bounding_box()
    }

    fn entity_type(&self) -> Type {
        self.button.entity_type()
    }
}

impl FlexItem for CharacterGui {
    fn set_position(&mut self, position: &Vector<f32>) {
        self.button.set_position(position);
        self.bars.set_position(position);
    }

    fn is_dirty(&mut self) -> bool {
        self.button.is_dirty()
    }

    fn set_focus(&mut self, focus: bool) {
        self.button.set_focus(focus);
    }

    fn has_focus(&self) -> bool {
        self.button.has_focus()
    }
}
