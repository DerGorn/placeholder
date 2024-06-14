use crate::create_name_struct;
use std::{fmt::Debug, time::Duration};
use threed::Vector;
use winit::event::KeyEvent;

use crate::game::Vertex;

use super::{
    ressource_descriptor::SpriteSheetName, sprite_sheet::SpriteSheet, BoundingBox, ExternalEvent,
    Index, SpritePosition,
};

create_name_struct!(EntityName);

pub trait EntityType: PartialEq + Debug {}

pub trait Entity<T: EntityType, E: ExternalEvent>: Debug + Send {
    fn update(&mut self, _entities: &Vec<&Box<dyn Entity<T, E>>>, _delta_t: &Duration) -> Vec<E> {
        vec![]
    }
    fn render(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        sprite_sheet: Option<&SpriteSheet>,
    );
    fn render_sprite(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<Index>,
        sprite_sheet: &SpriteSheet,
        sprite_position: &SpritePosition,
    ) {
        let bounding_box = self.bounding_box();
        let y = bounding_box.anchor.y;
        let x = bounding_box.anchor.x;
        let x_offset = bounding_box.size.width / 2.0;
        let y_offset = bounding_box.size.height / 2.0;
        let texture_coords = sprite_sheet.get_sprite_coordinates(sprite_position);
        let new_vertices = [
            Vertex::new(
                Vector::new(x - x_offset, y + y_offset, 0.0),
                &texture_coords[0],
                sprite_sheet.texture(),
            ),
            Vertex::new(
                Vector::new(x + x_offset, y + y_offset, 0.0),
                &texture_coords[1],
                sprite_sheet.texture(),
            ),
            Vertex::new(
                Vector::new(x + x_offset, y - y_offset, 0.0),
                &texture_coords[2],
                sprite_sheet.texture(),
            ),
            Vertex::new(
                Vector::new(x - x_offset, y - y_offset, 0.0),
                &texture_coords[3],
                sprite_sheet.texture(),
            ),
        ];
        let start_index = vertices.len() as u16;
        let new_indices = [
            start_index,
            start_index + 1,
            start_index + 2,
            start_index,
            start_index + 2,
            start_index + 3,
        ];
        vertices.extend_from_slice(&new_vertices);
        indices.extend_from_slice(&new_indices);
    }
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
