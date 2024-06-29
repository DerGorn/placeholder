use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, Entity, EntityName, SceneName, SpriteSheet},
};
use std::{fmt::Debug, time::Duration};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{animation::Animation, vertex::SimpleVertex, Event, Index, Type, UTIME};

pub enum TransitionTypes {
    BattleTransition,
}

fn lerp(start: Vec<Vector<f32>>, end: Vec<Vector<f32>>, steps: u16) -> Vec<Vec<Vector<f32>>> {
    let deltas = start
        .iter()
        .enumerate()
        .map(|(n, f)| (&end[n] - f) / (steps - 1) as f32)
        .collect::<Vec<_>>();
    let mut interpolations = Vec::new();
    for i in 0..steps {
        let step = start
            .iter()
            .enumerate()
            .map(|(n, f)| f + &deltas[n] * (i as f32))
            .collect::<Vec<_>>();
        interpolations.push(step);
    }
    interpolations
}

pub struct Transition {
    name: EntityName,
    animation: Animation<(Vec<SimpleVertex>, Vec<Index>)>,
    time: f32,
    transition_time: f32,
    running: bool,
}
impl Transition {
    pub fn new(transition_type: TransitionTypes, name: &str, transition_time: Duration) -> Self {
        let steps = 20;
        let frame_time = (transition_time.as_nanos() / (steps as u128)) as u64;
        let animation = match transition_type {
            TransitionTypes::BattleTransition => Animation::new(
                name.into(),
                lerp(
                    vec![
                        Vector::new(-0.05, 0.05, 0.0),
                        Vector::new(0.05, 0.05, 0.0),
                        Vector::new(0.05, -0.05, 0.0),
                        Vector::new(-0.05, -0.05, 0.0),
                    ],
                    vec![
                        Vector::new(-1.0, 1.0, 0.0),
                        Vector::new(1.0, 1.0, 0.0),
                        Vector::new(1.0, -1.0, 0.0),
                        Vector::new(-1.0, -1.0, 0.0),
                    ],
                    steps,
                )
                .iter()
                .map(|positions| {
                    (
                        Duration::from_nanos(frame_time),
                        (
                            positions
                                .iter()
                                .map(|p| SimpleVertex::new(p.clone()))
                                .collect::<Vec<_>>(),
                            vec![0, 1, 2, 0, 2, 3],
                        ),
                    )
                })
                .collect::<Vec<_>>(),
                true,
            ),
        };
        Transition {
            name: name.into(),
            animation,
            time: 0.0,
            transition_time: transition_time.as_secs_f32(),
            running: true,
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
        _sprite_sheet: Vec<&SpriteSheet>,
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
        _scene: &SceneName,
    ) -> Vec<Event> {
        if self.running {
            let end_reached = self.animation.update(delta_t);
            self.running = !end_reached;
            self.time = (self.time + delta_t.as_secs_f32()) % self.transition_time;
            if self.running {
                vec![Event::UpdateUniformBuffer(
                    UTIME.into(),
                    bytemuck::cast_slice(&[self.time / self.transition_time]).to_vec(),
                )]
            } else {
                vec![Event::AnimationEnded(self.name.clone())]
            }
        } else {
            vec![]
        }
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
