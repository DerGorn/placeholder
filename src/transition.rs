use std::{fmt::Debug, time::Duration};
use placeholder::{app::{IndexBuffer, VertexBuffer}, game_engine::{BoundingBox, Entity, EntityName, SpriteSheet}};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{animation::Animation, vertex::SimpleVertex, Event, Index, Type};

pub struct Transition {
    pub name: EntityName,
    pub animation: Animation<(Vec<SimpleVertex>, Vec<Index>)>,
}
impl Debug for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transition")
            .field("name", &self.name)
            .finish()
    }
}
impl Entity<Type, Event> for Transition {
    fn render(
        &self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        _sprite_sheet: Option<&SpriteSheet>,
    ) {
        let (new_vertices, new_indices) = self.animation.keyframe();
        let start_index = vertices.len() as u16;
        vertices.extend_from_slice(new_vertices);
        indices.extend_from_slice(
            new_indices
                .iter()
                .map(|i| i + start_index)
                .collect::<Vec<_>>()
                .as_slice(),
        );
    }

    fn update(
        &mut self,
        _entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &Duration,
    ) -> Vec<Event> {
        self.animation.update(delta_t);
        vec![]
    }

    fn name(&self) -> &EntityName {
        &self.name
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: Vector::scalar(0.0),
            size: PhysicalSize::new(1e5, 1e5),
        }
    }

    fn entity_type(&self) -> Type {
        Type::Background
    }
}


