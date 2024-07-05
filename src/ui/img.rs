use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, Entity, EntityName, SpritePosition, SpriteSheet, SpriteSheetName},
};
use std::fmt::Debug;
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{vertex::render_ui_sprite, Event, Type};

use super::FlexItem;

pub struct Image {
    dimensions: PhysicalSize<u16>,
    position: Vector<f32>,
    name: EntityName,
    image: (SpriteSheetName, SpritePosition),
    is_dirty: bool,
}
impl Image {
    pub fn new(
        name: EntityName,
        dimensions: PhysicalSize<u16>,
        position: Vector<f32>,
        image: (SpriteSheetName, SpritePosition),
    ) -> Self {
        Self {
            dimensions,
            position,
            name,
            image,
            is_dirty: true,
        }
    }
}
impl Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("name", &self.name)
            .field("image", &self.image.0)
            .finish()
    }
}
impl Entity<Type, Event> for Image {
    fn render(
        &mut self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        sprite_sheet: Vec<Option<&SpriteSheet>>,
    ) {
        if let Some(sprite_sheet) = sprite_sheet[0] {
            render_ui_sprite(
                &self.bounding_box(),
                vertices,
                indices,
                sprite_sheet,
                &self.image.1,
            )
        }
    }
    fn sprite_sheets(&self) -> Vec<&SpriteSheetName> {
        vec![&self.image.0]
    }
    fn entity_type(&self) -> Type {
        Type::Menu
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn bounding_box(&self) -> placeholder::game_engine::BoundingBox {
        BoundingBox {
            anchor: self.position.clone(),
            size: PhysicalSize::new(self.dimensions.width as f32, self.dimensions.height as f32),
        }
    }
}
impl FlexItem for Image {
    fn position_mut(&mut self) -> &mut Vector<f32> {
        &mut self.position
    }

    fn is_dirty(&mut self) -> bool {
        let dirt = self.is_dirty;
        self.is_dirty = false;
        dirt
    }
}
