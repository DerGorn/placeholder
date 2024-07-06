use std::fmt::Debug;

use log::warn;
use placeholder::game_engine::{BoundingBox, Entity, EntityName, SpritePosition, SpriteSheetName};
use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{impl_flex_struct, vertex::render_ui_sprite, Event, Type};

use super::{Alignment, FlexDirection, FlexItem};

mod button;
pub use button::Button;

pub struct FlexButtonLine {
    flex_direction: FlexDirection,
    /// Alignment of children orthogonal to the flex direction
    align_content: Alignment,
    background_image: Option<(SpriteSheetName, SpritePosition)>,
    gap: f32,
    dimensions: PhysicalSize<u16>,
    position: Vector<f32>,
    children: Vec<Box<Button>>,
    name: EntityName,
    shrink_to_content: bool,
    number_of_sprites: Vec<usize>,
    is_dirty: bool,
    focused_child: usize,
}
impl FlexButtonLine {
    pub fn new(
        flex_direction: FlexDirection,
        align_content: Alignment,
        background_image: Option<(SpriteSheetName, SpritePosition)>,
        gap: f32,
        shrink_to_content: bool,
        dimensions: PhysicalSize<u16>,
        position: Vector<f32>,
        name: EntityName,
        mut children: Vec<Box<Button>>,
    ) -> Self {
        let number_of_sprites = children.iter().map(|x| x.sprite_sheets().len()).collect();
        if children.len() > 0 {
            children[0].set_focus(true);
        }
        Self {
            flex_direction,
            align_content,
            background_image,
            gap,
            dimensions,
            position,
            children,
            name,
            shrink_to_content,
            number_of_sprites,
            is_dirty: true,
            focused_child: 0,
        }
    }

    fn set_focus(&mut self, focused_child: usize) {
        if focused_child < self.children.len() {
            self.children[self.focused_child].set_focus(false);
            self.focused_child = focused_child;
            self.children[self.focused_child].set_focus(true);
        } else {
            warn!("Trying to focus non existing button {}", focused_child);
        }
    }

    fn render_background(
        &self,
        vertices: &mut placeholder::app::VertexBuffer,
        indices: &mut placeholder::app::IndexBuffer,
        sprite_sheet: &[Option<&placeholder::game_engine::SpriteSheet>],
        index: &mut usize,
    ) {
        if let Some((background, sprite_position)) = &self.background_image {
            if let Some(sprite_sheet) = sprite_sheet.get(0).expect("Got no option in sprite_sheets")
            {
                *index += 1;
                render_ui_sprite(
                    &self.bounding_box(),
                    vertices,
                    indices,
                    sprite_sheet,
                    sprite_position,
                    None,
                )
            } else {
                log::warn!(
                    "No sprite sheet for background image {:?} of FlexBox {:?}",
                    background,
                    self.name
                );
            }
        }
    }
}
impl_flex_struct!(FlexButtonLine);
impl Debug for FlexButtonLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlexButtonLine")
            .field("name", &self.name)
            .field("position", &self.position)
            .finish()
    }
}
impl Entity<Type, Event> for FlexButtonLine {
    fn handle_key_input(&mut self, input: &winit::event::KeyEvent) -> Vec<Event> {
        if input.state == winit::event::ElementState::Pressed {
            let selection_change = match self.flex_direction {
                FlexDirection::Y => match input.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => 1,
                    PhysicalKey::Code(KeyCode::KeyS) => -1,
                    _ => 0,
                },
                FlexDirection::X => match input.physical_key {
                    PhysicalKey::Code(KeyCode::KeyD) => 1,
                    PhysicalKey::Code(KeyCode::KeyA) => -1,
                    _ => 0,
                },
            };
            let new_focus =
                (self.focused_child as i32 + selection_change + self.children.len() as i32)
                    % self.children.len() as i32;
            self.set_focus(new_focus as usize);
        }
        self.flex_handle_key_input(input)
    }
    fn update(
        &mut self,
        entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &std::time::Duration,
        scene: &placeholder::game_engine::SceneName,
    ) -> Vec<Event> {
        self.flex_update(entities, delta_t, scene)
    }
    fn render(
        &mut self,
        vertices: &mut placeholder::app::VertexBuffer,
        indices: &mut placeholder::app::IndexBuffer,
        sprite_sheet: Vec<Option<&placeholder::game_engine::SpriteSheet>>,
    ) {
        self.flex_render(vertices, indices, sprite_sheet)
    }
    fn sprite_sheets(&self) -> Vec<&placeholder::game_engine::SpriteSheetName> {
        self.flex_sprite_sheets()
    }
    fn entity_type(&self) -> Type {
        Type::Menu
    }
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: self.position.clone(),
            size: PhysicalSize::new(self.dimensions.width as f32, self.dimensions.height as f32),
        }
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
}
impl FlexItem for FlexButtonLine {
    fn position_mut(&mut self) -> &mut Vector<f32> {
        &mut self.position
    }

    fn is_dirty(&mut self) -> bool {
        let dirt = self.is_dirty;
        self.is_dirty = false;
        dirt
    }
}
