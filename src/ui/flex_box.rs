use placeholder::game_engine::{BoundingBox, Entity, EntityName, SpriteSheetName};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{Event, Type};

#[derive(Debug)]
pub enum FlexDirection {
    X,
    Y,
}

pub trait FlexItem: Entity<Type, Event> {
    fn set_position(&mut self, position: Vector<f32>);
}

#[derive(Debug)]
pub struct FlexBox {
    pub flex_direction: FlexDirection,
    pub dimensions: PhysicalSize<u16>,
    pub position: Vector<f32>,
    pub children: Vec<Box<dyn FlexItem>>,
    pub name: EntityName,
}
impl Entity<Type, Event> for FlexBox {
    fn render(
            &self,
            vertices: &mut placeholder::app::VertexBuffer,
            indices: &mut placeholder::app::IndexBuffer,
            sprite_sheet: Vec<&placeholder::game_engine::SpriteSheet>,
        ) {
        for item in self.children.iter() {
            item.render(vertices, indices, sprite_sheet.clone())
        }
    }
    fn handle_key_input(&mut self, input: &winit::event::KeyEvent) {
        for item in self.children.iter_mut() {
            item.handle_key_input(input)
        }
    }
    fn sprite_sheets(&self) -> Vec<&SpriteSheetName> {
        self.children.iter().map(|item| item.sprite_sheets()).flatten().collect()
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: self.position.clone(),
            size: PhysicalSize::new(self.dimensions.width as f32, self.dimensions.height as f32),
        }
    }
    fn entity_type(&self) -> Type {
        Type::Menu
    }
}
