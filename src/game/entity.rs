use std::fmt::Debug;
use placeholder::create_name_struct;
use threed::Vector;
use winit::event::KeyEvent;

use crate::vertex::Vertex;

use super::{ressource_descriptor::SpriteSheetName, sprite_sheet::SpriteSheet, Index};

create_name_struct!(EntityName);

pub trait Entity: Debug {
    fn update(&mut self);
    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        sprite_sheet: &SpriteSheet,
    );
    fn sprite_sheet(&self) -> &SpriteSheetName;
    fn handle_key_input(&mut self, input: &KeyEvent);
    fn z(&self) -> f32 {
        0.0
    }
    fn name(&self) -> &EntityName;
    fn position(&self) -> Vector<f32>;
}
