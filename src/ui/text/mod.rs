use log::warn;
use placeholder::game_engine::{BoundingBox, Entity, EntityName, SpriteSheet, SpriteSheetName};
use std::fmt::Debug;
// use std::cell::RefCell;
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{color::Color, ui::FlexItem, Event, Type, FONT};

use self::font_manager::render_character;

mod font_manager;

pub struct Text {
    text: String,
    color: Color,
    name: EntityName,
    size: PhysicalSize<u16>,
    max_size: PhysicalSize<u16>,
    position: Vector<f32>,
    sprite_sheet: SpriteSheetName,
    font_size: u8,
    fit_to_content: bool,
}
impl Text {
    pub fn new(
        text: String,
        color: Color,
        name: EntityName,
        size: PhysicalSize<u16>,
        position: Vector<f32>,
        font_size: u8,
        fit_to_content: bool,
    ) -> Self {
        Self {
            text,
            color,
            name,
            max_size: size.clone(),
            size,
            position,
            sprite_sheet: FONT.into(),
            font_size,
            fit_to_content,
        }
    }
}
impl Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("text", &self.text)
            .field("position", &self.position)
            .finish()
    }
}
impl Entity<Type, Event> for Text {
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
        sprite_sheet: Vec<Option<&SpriteSheet>>,
    ) {
        let mut text_width = f32::NEG_INFINITY;
        let font = if let Some(ss) = sprite_sheet[0] {
            ss
        } else {
            return;
        };
        let mut char_y: u16 = 0;
        let anchor = &self.position
            + Vector::new(
                -(self.size.width as f32 - self.font_size as f32) / 2.0,
                (self.size.height as f32 - self.font_size as f32) / 2.0,
                0.0,
            );
        let mut char_bounding_box = BoundingBox {
            anchor: anchor.clone(),
            size: PhysicalSize::new(self.font_size as f32, self.font_size as f32),
        };
        let width = self.position.x + (self.size.width as f32 - self.font_size as f32) / 2.0;
        let height = self.size.height / self.font_size as u16;
        for s in self.text.chars() {
            let new_line = s == '\n';
            if new_line || char_bounding_box.anchor.x >= width {
                char_y += 1;
                if char_y >= height {
                    warn!("Text too long for bounding box");
                    break;
                }
                if char_bounding_box.anchor.x >= text_width {
                    text_width = char_bounding_box.anchor.x;
                }
                char_bounding_box.anchor.x = anchor.x;
                char_bounding_box.anchor.y -= self.font_size as f32;
                if new_line {
                    continue;
                }
            }
            let character_width = render_character(s, &self.color, &char_bounding_box, vertices, indices, font);
            char_bounding_box.anchor.x += character_width;
        }
        if char_bounding_box.anchor.x >= text_width {
            text_width = char_bounding_box.anchor.x;
        }
        if self.fit_to_content {
            let height = ((char_y + 1) * self.font_size as u16).min(self.max_size.height);
            let width =
                ((text_width - anchor.x) as u16 + self.font_size as u16).min(self.max_size.width);
            self.size.width = width;
            self.size.height = height;
        }
        println!("Text: {:?}", self.text);
        println!("vertices: {:?}", vertices);
        println!("indices: {:?}", indices);
    }
    fn sprite_sheets(&self) -> Vec<&SpriteSheetName> {
        vec![&self.sprite_sheet]
    }
    fn entity_type(&self) -> Type {
        Type::Menu
    }
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: self.position.clone(),
            size: PhysicalSize {
                width: self.size.width as f32,
                height: self.size.height as f32,
            },
        }
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
}
impl FlexItem for Text {
    fn position_mut(&mut self) -> &mut Vector<f32> {
        &mut self.position
    }
}
