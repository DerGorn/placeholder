use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, Entity, EntityName, SpritePosition, SpriteSheet, SpriteSheetName},
    graphics::DEFAULT_TEXTURE,
};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{event::Event, vertex::render_ui_sprite, Type};

use super::{button_styles::ColorPair, FlexItem};

pub struct ProgressBar {
    max_value: u16,
    current_value: u16,
    dimensions: PhysicalSize<u16>,
    position: Vector<f32>,
    name: EntityName,
    colors: ColorPair,
    is_dirty: bool,
    sprite: SpriteSheetName,
    padding: u8,
}
impl ProgressBar {
    pub fn new(
        name: EntityName,
        dimensions: PhysicalSize<u16>,
        position: Vector<f32>,
        max_value: u16,
        current_value: u16,
        colors: ColorPair,
        padding: u8,
    ) -> Self {
        Self {
            max_value,
            current_value,
            dimensions,
            position,
            name,
            colors,
            padding,
            is_dirty: true,
            sprite: DEFAULT_TEXTURE.into(),
        }
    }

    pub fn set_value(&mut self, value: u16) {
        self.current_value = value;
    }
}
impl std::fmt::Debug for ProgressBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressBar")
            .field("name", &self.name)
            .field("max_value", &self.max_value)
            .field("current_value", &self.current_value)
            .finish()
    }
}
impl Entity<Type, Event> for ProgressBar {
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
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        sprite_sheet: Vec<Option<&SpriteSheet>>,
    ) {
        if let Some(sprite_sheet) = sprite_sheet[0] {
            let mut bounding_box = self.bounding_box();
            bounding_box.size.width -= 2.0 * self.padding as f32;
            bounding_box.size.height -= 2.0 * self.padding as f32;
            bounding_box.anchor.x += self.padding as f32;
            bounding_box.anchor.y += self.padding as f32;
            let sprite_position = SpritePosition::new(0, 0);
            render_ui_sprite(
                &bounding_box,
                vertices,
                indices,
                sprite_sheet,
                &sprite_position,
                Some(&self.colors.low),
            );
            let width_scale = if self.max_value == 0 {
                0.0
            } else {
                (self.current_value as f32 / self.max_value as f32)
                    .abs()
                    .min(1.0)
            };
            let offset = bounding_box.size.width * (1.0 - width_scale) / 2.0;
            bounding_box.size.width *= width_scale;
            bounding_box.anchor.x += offset;
            render_ui_sprite(
                &bounding_box,
                vertices,
                indices,
                sprite_sheet,
                &sprite_position,
                Some(&self.colors.high),
            );
        }
    }

    fn sprite_sheets(&self) -> Vec<&SpriteSheetName> {
        vec![&self.sprite]
    }

    fn name(&self) -> &EntityName {
        &self.name
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: &self.position - Vector::new(self.padding as f32, self.padding as f32, 0.0),
            size: PhysicalSize::new(
                self.dimensions.width as f32 + 2.0 * self.padding as f32,
                self.dimensions.height as f32 + 2.0 * self.padding as f32,
            ),
        }
    }

    fn entity_type(&self) -> Type {
        Type::Menu
    }
}
impl FlexItem for ProgressBar {
    fn is_dirty(&mut self) -> bool {
        let dirt = self.is_dirty;
        self.is_dirty = false;
        dirt
    }

    fn has_focus(&self) -> bool {
        false
    }

    fn set_focus(&mut self, _focus: bool) {}

    fn set_position(&mut self, position: &Vector<f32>) {
        self.is_dirty = true;
        self.position = position.clone();
    }
}
