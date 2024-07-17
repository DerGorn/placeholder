use log::warn;
use placeholder::game_engine::{BoundingBox, Entity, EntityName, SpriteSheet, SpriteSheetName};
use std::fmt::Debug;
// use std::cell::RefCell;
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{color::Color, ui::FlexItem, Event, Type, FONT};

use self::font_manager::render_character;

mod font_manager;

#[derive(Debug, Clone)]
pub struct FontSize(u8);
impl FontSize {
    pub fn new(size: u8) -> Self {
        if size & 1 == 1 {
            warn!("Font size must be even, got {}. Adding 1...", size);
            Self(size + 1)
        } else {
            Self(size)
        }
    }
}

pub struct Text {
    text: String,
    pub color: Color,
    name: EntityName,
    size: PhysicalSize<u16>,
    max_size: PhysicalSize<u16>,
    position: Vector<f32>,
    sprite_sheet: SpriteSheetName,
    font_size: FontSize,
    fit_to_content: bool,
    is_dirty: bool,
}
impl Text {
    pub fn new(
        mut text: String,
        color: Color,
        name: EntityName,
        size: PhysicalSize<u16>,
        position: Vector<f32>,
        font_size: FontSize,
        fit_to_content: bool,
    ) -> Self {
        text += " ";
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
            is_dirty: true,
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
        let color = &self.color;
        let font_size = self.font_size.0 as f32;
        let mut text_width = f32::NEG_INFINITY;
        let font = if let Some(ss) = sprite_sheet[0] {
            ss
        } else {
            return;
        };
        let mut char_y: u16 = 0;
        let anchor = &self.position
            + Vector::new(
                -(self.size.width as f32 - self.font_size.0 as f32) / 2.0,
                (self.size.height as f32 - self.font_size.0 as f32) / 2.0,
                0.0,
            );
        let mut char_bounding_box = BoundingBox {
            anchor: anchor.clone(),
            size: PhysicalSize::new(font_size, font_size),
        };
        let width = self.position.x + (self.size.width as f32) / 2.0;
        let height = self.size.height / font_size as u16;
        for chars in self.text.as_bytes().windows(2) {
            let current = chars[0];
            let next = chars[1];
            // if current == \n
            let new_line = current == 10;
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
                char_bounding_box.anchor.y -= font_size;
                if new_line {
                    continue;
                }
            }
            let character_width = render_character(
                current,
                next,
                &color,
                &char_bounding_box,
                vertices,
                indices,
                font,
            );
            char_bounding_box.anchor.x += character_width;
        }
        if char_bounding_box.anchor.x >= text_width {
            text_width = char_bounding_box.anchor.x;
        }
        if self.fit_to_content {
            let height = (((char_y as f32 + 1.25) * font_size) as u16).min(self.max_size.height);
            let width =
                ((text_width - anchor.x) as u16 + font_size as u16 / 4).min(self.max_size.width);
            if width != self.size.width || height != self.size.height {
                self.size.width = width;
                self.size.height = height;
                self.is_dirty = true;
            }
        }
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
    fn set_position(&mut self, position: &Vector<f32>) {
        self.position = position.clone();
    }
    fn is_dirty(&mut self) -> bool {
        let dirt = self.is_dirty;
        self.is_dirty = false;
        dirt
    }
    fn set_focus(&mut self, _focus: bool) {
    }
    fn has_focus(&self) -> bool {
        false
    }
}
