use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, Entity, EntityName, SpriteSheet},
};
use std::{fmt::Debug, time::Duration};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{animation::Animation, vertex::SimpleVertex, Event, Index, Type, UTIME};

pub enum TransitionTypes {
    BattleTransition,
}

pub struct Transition {
    pub name: EntityName,
    pub animation: Animation<(Vec<SimpleVertex>, Vec<Index>)>,
    time: f32,
}
impl Transition {
    pub fn new(transition_type: TransitionTypes, name: &str) -> Self {
        let animation = match transition_type {
            TransitionTypes::BattleTransition => Animation::new(
                name.into(),
                vec![
                    (
                        Duration::from_millis(24),
                        (
                            vec![
                                SimpleVertex::new(Vector::new(-0.5, 0.5, 0.0)),
                                SimpleVertex::new(Vector::new(0.5, 0.5, 0.0)),
                                SimpleVertex::new(Vector::new(0.5, -0.5, 0.0)),
                                SimpleVertex::new(Vector::new(-0.5, -0.5, 0.0)),
                            ],
                            vec![0, 1, 2, 0, 2, 3],
                        ),
                    ),
                    (
                        Duration::from_millis(24),
                        (
                            vec![
                                SimpleVertex::new(Vector::new(-0.75, 0.75, 0.0)),
                                SimpleVertex::new(Vector::new(0.75, 0.75, 0.0)),
                                SimpleVertex::new(Vector::new(0.75, -0.75, 0.0)),
                                SimpleVertex::new(Vector::new(-0.75, -0.75, 0.0)),
                            ],
                            vec![0, 1, 2, 0, 2, 3],
                        ),
                    ),
                    (
                        Duration::from_millis(24),
                        (
                            vec![
                                SimpleVertex::new(Vector::new(-1.0, 1.0, 0.0)),
                                SimpleVertex::new(Vector::new(1.0, 1.0, 0.0)),
                                SimpleVertex::new(Vector::new(1.0, -1.0, 0.0)),
                                SimpleVertex::new(Vector::new(-1.0, -1.0, 0.0)),
                            ],
                            vec![0, 1, 2, 0, 2, 3],
                        ),
                    ),
                    (
                        Duration::from_millis(24),
                        (
                            vec![
                                SimpleVertex::new(Vector::new(-0.75, 0.75, 0.0)),
                                SimpleVertex::new(Vector::new(0.75, 0.75, 0.0)),
                                SimpleVertex::new(Vector::new(0.75, -0.75, 0.0)),
                                SimpleVertex::new(Vector::new(-0.75, -0.75, 0.0)),
                            ],
                            vec![0, 1, 2, 0, 2, 3],
                        ),
                    ),
                ],
            ),
        };
        Transition {
            name: name.into(),
            animation,
            time: 0.0,
        }
    }
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
        self.time += delta_t.as_secs_f32() / 100.0;
        vec![Event::UpdateUniformBuffer(
            UTIME.into(),
            bytemuck::cast_slice(&[self.time]).to_vec(),
        )]
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
