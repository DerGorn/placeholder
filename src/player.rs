use std::{fmt::Debug, time::Duration};

use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{
        BoundingBox, Direction, Entity, EntityName, SceneName, SpritePosition, SpriteSheet,
        SpriteSheetName, VelocityController,
    },
};
use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{animation::Animation, vertex::render_sprite, Event, Type};

pub struct Player {
    pub name: EntityName,
    pub size: PhysicalSize<u16>,
    pub position: Vector<f32>,
    pub velocity: VelocityController,
    pub animation: Animation<SpritePosition>,
}
impl Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player")
            .field("z", &self.z())
            .field("sprite", &self.sprite_sheets())
            .finish()
    }
}
impl Entity<Type, Event> for Player {
    fn entity_type(&self) -> Type {
        Type::Player
    }
    fn update(
        &mut self,
        entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &Duration,
        _scene: &SceneName,
    ) -> Vec<Event> {
        self.position += self.velocity.get_velocity();
        let background = entities
            .iter()
            .filter(|e| e.entity_type() == Type::Background)
            .next()
            .expect("No Background found to restrict Playermovement");
        if let Some(new_position) = background
            .bounding_box()
            .clamp_box_inside(&self.bounding_box())
        {
            self.position = new_position;
        }
        let enemies = entities.iter().filter(|e| e.entity_type() == Type::Enemy);
        let own_bounding_box = self.bounding_box();
        for enemy in enemies {
            let bounding_box = enemy.bounding_box();
            if own_bounding_box.intersects(&bounding_box) {
                self.velocity.stop_movement();
                break;
            }
        }
        self.animation.update(delta_t);
        vec![]
    }

    fn name(&self) -> &EntityName {
        &self.name
    }

    fn position(&self) -> Vector<f32> {
        self.position.clone()
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: self.position.clone(),
            size: PhysicalSize::new(self.size.width as f32, self.size.height as f32),
        }
    }

    fn sprite_sheets(&self) -> Vec<&SpriteSheetName> {
        vec![&self.animation.sprite_sheet()]
    }

    fn z(&self) -> f32 {
        self.position.z
    }

    fn render(
        &mut self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        sprite_sheet: Vec<Option<&SpriteSheet>>,
    ) {
        if let Some(sprite_sheet) = sprite_sheet[0] {
            render_sprite(
                &self.bounding_box(),
                vertices,
                indices,
                sprite_sheet,
                self.animation.keyframe(),
            );
        }
    }

    fn handle_key_input(&mut self, input: &KeyEvent) {
        if input.state == winit::event::ElementState::Released {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.velocity.set_direction(Direction::Up, false);
                    // self.sprite.position = PLAYER_NEUTRAL;
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.velocity.set_direction(Direction::Left, false);
                    // self.sprite.position = PLAYER_NEUTRAL;
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.velocity.set_direction(Direction::Right, false);
                    // self.sprite.position = PLAYER_NEUTRAL;
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.velocity.set_direction(Direction::Down, false);
                    // self.sprite.position = PLAYER_NEUTRAL;
                }
                _ => {}
            }
        } else if input.state == winit::event::ElementState::Pressed {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.velocity.set_direction(Direction::Up, true);
                    // self.sprite.position = PLAYER_UP;
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.velocity.set_direction(Direction::Left, true);
                    // self.sprite.position = PLAYER_LEFT;
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.velocity.set_direction(Direction::Right, true);
                    // self.sprite.position = PLAYER_RIGHT;
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.velocity.set_direction(Direction::Down, true);
                    // self.sprite.position = PLAYER_DOWN;
                }
                _ => {}
            }
        } else {
            // self.sprite.position = PLAYER_NEUTRAL;
        }
    }
}
