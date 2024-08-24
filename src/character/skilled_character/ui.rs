use placeholder::game_engine::Entity;
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{
    character::Character,
    event::Event,
    ui::{Button, FlexItem, FlexProgressBarLine, ProgressBar},
    Type,
};

#[derive(Debug)]
pub struct CharacterGui {
    button: Box<Button>,
    bars: FlexProgressBarLine,
}
impl CharacterGui {
    pub fn new(button: Box<Button>, bars: Vec<Box<ProgressBar>>) -> Self {
        let bbox = button.bounding_box();
        Self {
            bars: FlexProgressBarLine::new(
                crate::ui::FlexDirection::Y,
                crate::ui::FlexOrigin::End,
                crate::ui::Alignment::End,
                None,
                10.0,
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

    pub fn set_content(&mut self, character: &Character) {
        self.button.set_content(character.to_string())
    }
}
impl Entity<Type, Event> for CharacterGui {
    fn update(
        &mut self,
        _entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        _delta_t: &std::time::Duration,
        _scene: &placeholder::game_engine::SceneName,
    ) -> Vec<Event> {
        vec![]
    }

    fn render(
        &mut self,
        vertices: &mut placeholder::app::VertexBuffer,
        indices: &mut placeholder::app::IndexBuffer,
        sprite_sheets: Vec<Option<&placeholder::game_engine::SpriteSheet>>,
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

    fn sprite_sheets(&self) -> Vec<&placeholder::game_engine::SpriteSheetName> {
        let mut sprites = self.button.sprite_sheets();
        sprites.append(&mut self.bars.sprite_sheets());
        sprites
    }

    fn handle_key_input(&mut self, input: &winit::event::KeyEvent) -> Vec<Event> {
        self.button.handle_key_input(input)
    }

    fn name(&self) -> &placeholder::game_engine::EntityName {
        self.button.name()
    }

    fn bounding_box(&self) -> placeholder::game_engine::BoundingBox {
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
