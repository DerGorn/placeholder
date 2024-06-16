use placeholder::{app::{IndexBuffer, VertexBuffer}, game_engine::{
    BoundingBox, Entity, EntityName, SpritePosition, SpriteSheet, SpriteSheetName,
}};
use std::fmt::Debug;
use threed::Vector;
use winit::{dpi::PhysicalSize, event::KeyEvent};

use crate::{vertex::render_sprite, Event, Type};

pub struct Background {
    pub name: EntityName,
    pub sprite_sheet: SpriteSheetName,
    pub size: PhysicalSize<u16>,
}
impl Debug for Background {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Background")
            .field("z", &self.z())
            .field("sprite", &self.sprite_sheet())
            .finish()
    }
}
impl Entity<Type, Event> for Background {
    fn entity_type(&self) -> Type {
        Type::Background
    }
    fn sprite_sheet(&self) -> Option<&SpriteSheetName> {
        Some(&self.sprite_sheet)
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn position(&self) -> Vector<f32> {
        Vector::new(0.0, 0.0, 0.0)
    }
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: Vector::new(0.0, 0.0, 0.0),
            size: PhysicalSize::new(self.size.width as f32, self.size.height as f32),
        }
    }
    fn z(&self) -> f32 {
        -1000.0
    }
    fn handle_key_input(&mut self, _input: &KeyEvent) {}
    fn render(
        &self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        sprite_sheet: Option<&SpriteSheet>,
    ) {
        if let Some(sprite_sheet) = sprite_sheet {
            render_sprite(&self.bounding_box(), vertices, indices, sprite_sheet, &SpritePosition::new(0, 0));
        }
    }
}
