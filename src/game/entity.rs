use placeholder::create_name_struct;
use std::fmt::Debug;
use threed::Vector;
use winit::event::KeyEvent;

use crate::vertex::Vertex;

use super::{ressource_descriptor::SpriteSheetName, sprite_sheet::SpriteSheet, BoundingBox, Index};

create_name_struct!(EntityName);

pub trait EntityType: PartialEq + Debug {}

pub trait Entity<T: EntityType>: Debug {
    fn update(&mut self, entities: &Vec<&Box<dyn Entity<T>>>);
    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        sprite_sheet: &SpriteSheet,
    );
    fn sprite_sheet(&self) -> &SpriteSheetName;
    fn handle_key_input(&mut self, input: &KeyEvent);
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
