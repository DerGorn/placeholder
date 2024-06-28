use log::warn;
use placeholder::game_engine::{
    BoundingBox, Entity, EntityName, SpriteSheet, SpriteSheetName,
};
use std::{cell::RefCell, fmt::Debug};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{Event, Type, FONT};

use self::font_manager::render_character;

mod font_manager;

pub struct Text {
    text: String,
    name: EntityName,
    size: PhysicalSize<u16>,
    position: Vector<f32>,
    sprite_sheet: SpriteSheetName,
    font_size: u8,
    first_render: RefCell<bool>,
}
impl Text {
    pub fn new(
        text: String,
        name: EntityName,
        size: PhysicalSize<u16>,
        position: Vector<f32>,
        font_size: u8,
    ) -> Self {
        Self {
            text,
            name,
            size,
            position,
            sprite_sheet: FONT.into(),
            font_size,
            first_render: true.into(),
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
        &self,
        vertices: &mut placeholder::app::VertexBuffer,
        indices: &mut placeholder::app::IndexBuffer,
        sprite_sheet: Option<&SpriteSheet>,
    ) {
        let font = if let Some(ss) = sprite_sheet {
            ss
        } else {
            return;
        };
        let mut char_x: u16 = 0;
        let mut char_y: u16 = 0;
        let anchor = &self.position
            + Vector::new(
                -(self.size.width as f32 / 2.0),
                self.size.height as f32 / 2.0,
                0.0,
            );
        let mut char_bounding_box = BoundingBox {
            anchor: anchor.clone(),
            size: PhysicalSize::new(self.font_size as f32, self.font_size as f32),
        };
        let width = self.size.width as f32;
        let height = self.size.height / self.font_size as u16;
        for s in self.text.chars() {
            if *self.first_render.borrow() {
                println!("Rendering text: {:?}", s);
                println!(
                    "At Position: ({}, {}) => {:?}",
                    char_x, char_y, char_bounding_box.anchor
                );
            }
            let new_line = s == '\n';
            if new_line || char_bounding_box.anchor.x >= width {
                char_x = 0;
                char_y += 1;
                if char_y >= height {
                    warn!("Text too long for bounding box");
                    break;
                }
                char_bounding_box.anchor.x = anchor.x;
                char_bounding_box.anchor.y -= self.font_size as f32;
                if new_line {
                    continue;
                }
            }
            let character_width = render_character(
                s,
                &char_bounding_box,
                vertices,
                indices,
                font,
                *self.first_render.borrow(),
            );
            char_x += 1;
            char_bounding_box.anchor.x += character_width;
        }
        self.first_render.replace(false);
    }
    fn sprite_sheet(&self) -> Option<&SpriteSheetName> {
        Some(&self.sprite_sheet)
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
