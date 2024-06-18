use std::{fmt::Debug, time::Duration};

use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{
        BoundingBox, Entity, EntityName, Scene, SpritePosition, SpriteSheet, SpriteSheetName,
    },
    graphics::ShaderDescriptor,
};
use threed::Vector;
use winit::{dpi::PhysicalSize, event::KeyEvent};

use crate::{
    animation::Animation,
    transition::Transition,
    vertex::{render_sprite, SimpleVertex},
    Event, Type, BATTLE_TRANSITION_SCENE, MAIN_WINDOW,
};

pub struct Enemy {
    pub name: EntityName,
    pub size: PhysicalSize<u16>,
    pub position: Vector<f32>,
    pub animation: Animation<SpritePosition>,
}
impl Debug for Enemy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enemy")
            .field("z", &self.z())
            .field("sprite", &self.sprite_sheet())
            .finish()
    }
}
impl Entity<Type, Event> for Enemy {
    fn update(
        &mut self,
        entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &Duration,
    ) -> Vec<Event> {
        self.animation.update(delta_t);
        let players = entities.iter().filter(|e| e.entity_type() == Type::Player);
        let own_bounding_box = self.bounding_box();
        for player in players {
            let bounding_box = player.bounding_box();
            if own_bounding_box.intersects(&bounding_box) {
                let shader_descriptor = ShaderDescriptor {
                    file: "res/shader/transition.wgsl",
                    vertex_shader: "vs_main",
                    fragment_shader: "fs_main",
                };
                return vec![Event::RequestNewScenes(vec![Scene {
                    name: BATTLE_TRANSITION_SCENE.into(),
                    render_scene: BATTLE_TRANSITION_SCENE.into(),
                    target_window: MAIN_WINDOW.into(),
                    z_index: 1,
                    entities: vec![Box::new(Transition {
                        name: "BattleTransition".into(),
                        animation: Animation::new(
                            "BattleTransition".into(),
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
                    })],
                    shader_descriptor,
                }])];
            }
        }
        vec![]
    }
    fn render(
        &self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        sprite_sheet: Option<&SpriteSheet>,
    ) {
        if let Some(sprite_sheet) = sprite_sheet {
            render_sprite(
                &self.bounding_box(),
                vertices,
                indices,
                sprite_sheet,
                self.animation.keyframe(),
            );
        }
    }
    fn sprite_sheet(&self) -> Option<&SpriteSheetName> {
        Some(&self.animation.sprite_sheet())
    }
    fn handle_key_input(&mut self, _input: &KeyEvent) {}
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: self.position.clone(),
            size: PhysicalSize::new(self.size.width as f32, self.size.height as f32),
        }
    }
    fn entity_type(&self) -> Type {
        Type::Enemy
    }
}
