use std::fmt::Debug;
use winit::{dpi::PhysicalSize, event::KeyEvent};

use crate::vertex::Vertex;

use super::{ressource_descriptor::SpriteSheetName, sprite_sheet::SpriteSheet, Index};

pub trait Entity: Debug {
    fn update(&mut self);
    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        window_size: &PhysicalSize<u32>,
        sprite_sheet: &SpriteSheet,
    );
    fn sprite_sheet(&self) -> &SpriteSheetName;
    fn handle_key_input(&mut self, input: &KeyEvent);
    fn z(&self) -> f32 {
        0.0
    }
}
