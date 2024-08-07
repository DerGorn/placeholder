use placeholder::game_engine::{BoundingBox, Entity, EntityName, SpritePosition, SpriteSheetName};
use std::fmt::{self, Debug};
use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{vertex::render_ui_sprite, Event, Type};

#[derive(Debug)]
pub enum FlexDirection {
    X,
    Y,
}
#[allow(dead_code)]
pub enum Alignment {
    Start,
    End,
    Center,
}
#[allow(dead_code)]
pub enum FlexOrigin {
    Start,
    End,
    Center,
}

pub trait FlexItem: Entity<Type, Event> {
    fn set_position(&mut self, position: &Vector<f32>);
    fn is_dirty(&mut self) -> bool;
    fn set_focus(&mut self, focus: bool);
    fn has_focus(&self) -> bool;
}
#[macro_export]
macro_rules! impl_flex_struct {
    ($name: ident <$generic: ident : $restriction: tt>) => {
        impl<$generic: $restriction> $name<$generic> {
            impl_flex_struct!();
        }
    };
    ($name: ident) => {
        impl $name {
            impl_flex_struct!();
        }
    };
    () => {
        fn flex(&mut self) {
            let boxes = self.children.iter().map(|x| x.bounding_box());

            let flex_origin = &self.position
                + Vector::new(
                    -(self.dimensions.width as f32) / 2.0 - self.gap,
                    self.dimensions.height as f32 / 2.0 + self.gap,
                    0.0,
                );
            let mut flex_points: Vec<Vector<f32>> = vec![flex_origin];
            let mut alignment_points: Vec<Vector<f32>> = vec![];
            let mut total_width = 0.0;
            let mut total_height = 0.0;
            for bbox in boxes {
                let (offset, alignment_offset) = match self.flex_direction {
                    FlexDirection::X => {
                        if bbox.size.height > total_height {
                            total_height = bbox.size.height;
                        }
                        total_width += bbox.size.width;
                        (
                            Vector::new(-bbox.size.width / 2.0, 0.0, 0.0),
                            Vector::new(0.0, bbox.size.height / 2.0, 0.0),
                        )
                    }
                    FlexDirection::Y => {
                        if bbox.size.width > total_width {
                            total_width = bbox.size.width;
                        }
                        total_height += bbox.size.height;
                        (
                            Vector::new(0.0, bbox.size.height / 2.0, 0.0),
                            Vector::new(-bbox.size.width / 2.0, 0.0, 0.0),
                        )
                    }
                };

                let start = &bbox.anchor + &offset;
                let end = &bbox.anchor - offset;
                let overlap = if let Some(last) = flex_points.last() {
                    match self.flex_direction {
                        FlexDirection::X => Vector::new(last.x - start.x + self.gap, 0.0, 0.0),
                        FlexDirection::Y => Vector::new(0.0, last.y - start.y - self.gap, 0.0),
                    }
                } else {
                    Vector::scalar(0.0)
                };

                flex_points.push(start + &overlap);
                flex_points.push(end + &overlap);

                let alignment_start = &bbox.anchor - &alignment_offset;
                let alignment_end = &bbox.anchor + alignment_offset;

                alignment_points.push(alignment_start);
                alignment_points.push(alignment_end);
            }
            flex_points.remove(0);
            let gap_size = self.gap as f32 * (self.children.len() as f32 - 1.0);
            let flex_offset = match self.flex_origin {
                FlexOrigin::Start => Vector::scalar(0.0),
                FlexOrigin::End => match self.flex_direction {
                    FlexDirection::X => Vector::new(
                        -total_width - gap_size + self.dimensions.width as f32,
                        0.0,
                        0.0,
                    ),
                    FlexDirection::Y => Vector::new(
                        0.0,
                        total_height + gap_size - self.dimensions.height as f32,
                        0.0,
                    ),
                },
                FlexOrigin::Center => match self.flex_direction {
                    FlexDirection::X => Vector::new(
                        (-total_width - gap_size + self.dimensions.width as f32) / 2.0,
                        0.0,
                        0.0,
                    ),
                    FlexDirection::Y => Vector::new(
                        0.0,
                        (total_height + gap_size - self.dimensions.height as f32) / 2.0,
                        0.0,
                    ),
                },
            };
            for i in 0..self.children.len() {
                let mut position = self.children[i].position();
                let flex_start = &flex_points[i * 2];
                let flex_end = &flex_points[i * 2 + 1];
                let flex_position = (flex_start + flex_end) / 2.0 + &flex_offset;
                match self.flex_direction {
                    FlexDirection::X => {
                        position.x = flex_position.x;
                    }
                    FlexDirection::Y => {
                        position.y = flex_position.y;
                    }
                }

                let align_start = &alignment_points[i * 2];
                let align_end = &alignment_points[i * 2 + 1];
                match self.align_content {
                    Alignment::Start => match self.flex_direction {
                        FlexDirection::X => {
                            position.y +=
                                self.position.y + self.dimensions.height as f32 / 2.0 - align_end.y;
                        }
                        FlexDirection::Y => {
                            position.x +=
                                self.position.x - self.dimensions.width as f32 / 2.0 - align_end.x;
                        }
                    },
                    Alignment::End => match self.flex_direction {
                        FlexDirection::X => {
                            position.y += self.position.y
                                - self.dimensions.height as f32 / 2.0
                                - align_start.y;
                        }
                        FlexDirection::Y => {
                            position.x += self.position.x + self.dimensions.width as f32 / 2.0
                                - align_start.x;
                        }
                    },
                    Alignment::Center => match self.flex_direction {
                        FlexDirection::X => position.y = self.position.y,
                        FlexDirection::Y => {
                            position.x = self.position.x;
                        }
                    },
                }
                self.children[i].set_position(&position);
            }
            if self.shrink_to_content {
                let new_dimensions = match self.flex_direction {
                    FlexDirection::X => winit::dpi::PhysicalSize::new(
                        total_width as u16 + gap_size as u16,
                        total_height as u16,
                    ),
                    FlexDirection::Y => winit::dpi::PhysicalSize::new(
                        total_width as u16,
                        total_height as u16 + gap_size as u16,
                    ),
                };
                if new_dimensions != self.dimensions {
                    self.dimensions = new_dimensions;
                    self.is_dirty = true;
                } else {
                    self.is_dirty = false;
                }
            } else {
                self.is_dirty = false;
            }
        }
        fn flex_update(
            &mut self,
            entities: &Vec<&Box<dyn Entity<Type, Event>>>,
            delta_t: &std::time::Duration,
            scene: &placeholder::game_engine::SceneName,
        ) -> Vec<Event> {
            if self.children.iter_mut().any(|c| c.is_dirty()) || self.is_dirty {
                self.flex();
            }
            for child in &mut self.children {
                child.update(entities, delta_t, scene);
            }
            vec![]
        }
        fn flex_render(
            &mut self,
            vertices: &mut placeholder::app::VertexBuffer,
            indices: &mut placeholder::app::IndexBuffer,
            sprite_sheet: Vec<Option<&placeholder::game_engine::SpriteSheet>>,
        ) {
            let mut index = 0;
            // $crate::vertex::render_ui_box_border(
            //     &self.bounding_box(),
            //     vertices,
            //     indices,
            //     4.0,
            //     &$crate::color::Color::from_str("white"),
            // );
            self.render_background(vertices, indices, &sprite_sheet, &mut index);
            for i in 0..self.children.len() {
                let item = &mut self.children[i];
                let number_of_sprites = self.number_of_sprites[i];
                let sprite_sheet = sprite_sheet
                    .get(index..index + number_of_sprites)
                    .expect("Got no option in sprite_sheets");
                index += number_of_sprites;
                item.render(vertices, indices, sprite_sheet.to_vec())
            }
        }
        fn flex_sprite_sheets(&self) -> Vec<&SpriteSheetName> {
            let mut sprite_sheets = vec![];
            if let Some((sprite_sheet, _)) = &self.background_image {
                sprite_sheets.push(sprite_sheet)
            }
            sprite_sheets.extend(
                self.children
                    .iter()
                    .map(|item| item.sprite_sheets())
                    .flatten(),
            );
            sprite_sheets
        }
        fn flex_handle_key_input(&mut self, input: &winit::event::KeyEvent) -> Vec<Event> {
            let mut events = vec![];
            for item in self.children.iter_mut() {
                events.append(&mut item.handle_key_input(input));
            }
            events
        }
    };
}

pub struct FlexBox {
    flex_direction: FlexDirection,
    flex_origin: FlexOrigin,
    /// Alignment of children orthogonal to the flex direction
    align_content: Alignment,
    background_image: Option<(SpriteSheetName, SpritePosition)>,
    gap: f32,
    dimensions: PhysicalSize<u16>,
    position: Vector<f32>,
    pub children: Vec<Box<dyn FlexItem>>,
    name: EntityName,
    shrink_to_content: bool,
    number_of_sprites: Vec<usize>,
    is_dirty: bool,
}
impl_flex_struct!(FlexBox);
impl FlexBox {
    pub fn new(
        flex_direction: FlexDirection,
        flex_origin: FlexOrigin,
        align_content: Alignment,
        background_image: Option<(SpriteSheetName, SpritePosition)>,
        gap: f32,
        shrink_to_content: bool,
        dimensions: PhysicalSize<u16>,
        position: Vector<f32>,
        name: EntityName,
        children: Vec<Box<dyn FlexItem>>,
    ) -> Self {
        let number_of_sprites = children.iter().map(|x| x.sprite_sheets().len()).collect();
        Self {
            flex_direction,
            flex_origin,
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
impl Debug for FlexBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlexBox")
            .field("flex_direction", &self.flex_direction)
            .field("children", &self.children)
            .field("name", &self.name)
            .finish()
    }
}
impl Entity<Type, Event> for FlexBox {
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
    fn handle_key_input(&mut self, input: &winit::event::KeyEvent) -> Vec<Event> {
        self.flex_handle_key_input(input)
    }
    fn sprite_sheets(&self) -> Vec<&SpriteSheetName> {
        self.flex_sprite_sheets()
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: self.position.clone(),
            size: PhysicalSize::new(self.dimensions.width as f32, self.dimensions.height as f32),
        }
    }
    fn entity_type(&self) -> Type {
        Type::Menu
    }
    fn delete_child_entity(&mut self, name: &EntityName) {
        self.children.retain(|child| child.name() != name);
        for child in &mut self.children {
            child.delete_child_entity(name);
        }
    }
}
impl FlexItem for FlexBox {
    fn set_position(&mut self, position: &Vector<f32>) {
        if self.position != *position {
            self.is_dirty = true
        }
        self.position = position.clone();
    }
    fn is_dirty(&mut self) -> bool {
        let dirt = self.is_dirty;
        self.is_dirty = false;
        dirt
    }
    fn set_focus(&mut self, _focus: bool) {}
    fn has_focus(&self) -> bool {
        false
    }
}
