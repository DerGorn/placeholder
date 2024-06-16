use crate::{
    app::{IndexBuffer, VertexBuffer},
    create_name_struct,
};
use std::{fmt::Debug, time::Duration};
use threed::Vector;
use winit::event::KeyEvent;

use super::{
    ressource_descriptor::SpriteSheetName, sprite_sheet::SpriteSheet, BoundingBox, ExternalEvent,
};

create_name_struct!(EntityName);

pub trait EntityType: PartialEq + Debug {}

pub trait Entity<T: EntityType, E: ExternalEvent>: Debug + Send {
    fn update(&mut self, _entities: &Vec<&Box<dyn Entity<T, E>>>, _delta_t: &Duration) -> Vec<E> {
        vec![]
    }
    fn render(
        &self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        sprite_sheet: Option<&SpriteSheet>,
    );
   fn sprite_sheet(&self) -> Option<&SpriteSheetName> {
        None
    }
    fn handle_key_input(&mut self, _input: &KeyEvent) {}
    fn name(&self) -> &EntityName;
    fn bounding_box(&self) -> BoundingBox;
    fn entity_type(&self) -> T;

    fn z(&self) -> f32 {
        self.position().z
    }
    fn position(&self) -> Vector<f32> {
        self.bounding_box().anchor
    }
}
