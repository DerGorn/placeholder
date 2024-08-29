use std::{fmt::Debug, time::Duration};

use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{
        BoundingBox, Entity, EntityName, SceneName, SpritePosition, SpriteSheet, SpriteSheetName,
    },
};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{animation::Animation, vertex::render_sprite, EnemyType, Event, Type};

pub struct Enemy {
    pub name: EntityName,
    pub size: PhysicalSize<u16>,
    pub position: Vector<f32>,
    pub animation: Animation<SpritePosition>,
    pub enemy_type: EnemyType,
    pub sprite_sheet: SpriteSheetName,
}
impl Debug for Enemy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enemy")
            .field("z", &self.z())
            .field("sprite", &self.sprite_sheets())
            .finish()
    }
}
impl Entity<Type, Event> for Enemy {
    fn update(
        &mut self,
        entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &Duration,
        scene: &SceneName,
    ) -> Vec<Event> {
        self.animation.update(delta_t);
        let players = entities.iter().filter(|e| e.entity_type() == Type::Player);
        let own_bounding_box = self.bounding_box();
        let mut start_fight = false;
        for player in players {
            let bounding_box = player.bounding_box();
            if own_bounding_box.intersects(&bounding_box) {
                start_fight = true;
                break;
            }
        }
        if start_fight {
            vec![Event::InitiateBattle(
                self.enemy_type.clone(),
                self.name.clone(),
                scene.clone(),
            )]
        } else {
            vec![]
        }
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
    fn sprite_sheets(&self) -> Vec<&SpriteSheetName> {
        vec![&self.sprite_sheet]
    }
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
