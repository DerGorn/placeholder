use std::time::Duration;

use ferride_core::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, Entity, EntityName, SpritePosition, SpriteSheet, SpriteSheetName},
    graphics::DEFAULT_TEXTURE,
    reexports::winit::PhysicalSize,
};
use threed::Vector;

use crate::{
    animation::Animation,
    color::Color,
    event::Event,
    vertex::{render_ui_box_border, render_ui_sprite},
    Type, TARGET_FPS,
};

use super::{button_styles::ColorPair, FlexItem, Padding};

const PROGRESS_BAR_ANIMATION_COLOR: Color = Color::new_rgba(255, 255, 255, 255);
const ANIMATION_STEPS: u16 = 30;
const ANIMATION_DURATION: Duration =
    Duration::from_millis(1000 / TARGET_FPS as u64 * ANIMATION_STEPS as u64);

pub struct ProgressBar {
    max_value: f32,
    current_value: f32,
    dimensions: PhysicalSize<u16>,
    position: Vector<f32>,
    name: EntityName,
    colors: ColorPair,
    is_dirty: bool,
    sprite: SpriteSheetName,
    padding: Padding,
    animation: Animation<f32>,
}
impl ProgressBar {
    pub fn new(
        name: EntityName,
        dimensions: PhysicalSize<u16>,
        position: Vector<f32>,
        max_value: u16,
        current_value: u16,
        colors: ColorPair,
        padding: Padding,
    ) -> Self {
        Self {
            max_value: max_value as f32,
            current_value: current_value as f32,
            dimensions,
            position,
            name,
            colors,
            padding,
            is_dirty: true,
            sprite: DEFAULT_TEXTURE.into(),
            animation: Animation::new(vec![(Duration::from_millis(0), current_value as f32)], true),
        }
    }

    /// returns whether the value has changed
    pub fn set_value(&mut self, value: u16) -> bool {
        if value as f32 == self.current_value {
            return false;
        }
        self.animation = Animation::lerp(
            *self.animation.keyframe(),
            value as f32,
            ANIMATION_STEPS,
            ANIMATION_DURATION,
            &|a| a,
            true,
        );
        self.current_value = value as f32;
        true
    }
}
impl std::fmt::Debug for ProgressBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressBar")
            .field("name", &self.name)
            .field("max_value", &self.max_value)
            .field("current_value", &self.current_value)
            .finish()
    }
}
fn render_bar_part(
    max_value: f32,
    current_value: f32,
    vertices: &mut VertexBuffer,
    indices: &mut IndexBuffer,
    sprite_sheet: &SpriteSheet,
    sprite_position: &SpritePosition,
    bounding_box: &BoundingBox,
    color: &Color,
) {
    let mut bounding_box = BoundingBox {
        anchor: bounding_box.anchor.clone(),
        size: bounding_box.size,
    };
    let width_scale = if max_value == 0.0 {
        0.0
    } else {
        (current_value / max_value).abs().min(1.0)
    };
    let offset = bounding_box.size.width * (1.0 - width_scale) / 2.0;
    bounding_box.size.width *= width_scale;
    bounding_box.anchor.x += offset;
    render_ui_sprite(
        &bounding_box,
        vertices,
        indices,
        sprite_sheet,
        &sprite_position,
        Some(color),
    );
}

impl Entity<Type, Event> for ProgressBar {
    fn update(
        &mut self,
        _entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &std::time::Duration,
        _scene: &ferride_core::game_engine::SceneName,
    ) -> Vec<Event> {
        if self.animation.update(delta_t) {
            vec![Event::AnimationEnded(self.name.clone())]
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
            let animation_value = *self.animation.keyframe();
            let mut bounding_box = self.bounding_box();
            bounding_box.size.width -= self.padding.left as f32 + self.padding.right as f32;
            bounding_box.size.height -= self.padding.up as f32 + self.padding.down as f32;
            bounding_box.anchor.x -= (self.padding.right as f32 - self.padding.left as f32) / 2.0;
            bounding_box.anchor.y -= (self.padding.up as f32 - self.padding.down as f32) / 2.0;
            let sprite_position = SpritePosition::new(0, 0);
            let border_thickness = bounding_box.size.width.min(bounding_box.size.height) / 10.0;
            bounding_box.size.height -= 2.0 * border_thickness;
            bounding_box.size.width -= 2.0 * border_thickness;
            render_ui_sprite(
                &bounding_box,
                vertices,
                indices,
                sprite_sheet,
                &sprite_position,
                Some(&self.colors.low),
            );
            render_ui_box_border(
                &bounding_box,
                vertices,
                indices,
                border_thickness,
                &self.colors.high,
            );
            let animation_differential = animation_value - self.current_value;
            let (value_color, animation_color) = if animation_differential <= 0.0 {
                (&PROGRESS_BAR_ANIMATION_COLOR, &self.colors.high)
            } else {
                (&self.colors.high, &PROGRESS_BAR_ANIMATION_COLOR)
            };
            if animation_differential >= 1e-4 {
                render_bar_part(
                    self.max_value,
                    animation_value,
                    vertices,
                    indices,
                    sprite_sheet,
                    &sprite_position,
                    &bounding_box,
                    animation_color,
                );
            }
            render_bar_part(
                self.max_value,
                self.current_value,
                vertices,
                indices,
                sprite_sheet,
                &sprite_position,
                &bounding_box,
                value_color,
            );
            if animation_differential <= 1e-4 {
                render_bar_part(
                    self.max_value,
                    animation_value,
                    vertices,
                    indices,
                    sprite_sheet,
                    &sprite_position,
                    &bounding_box,
                    animation_color,
                );
            }
        }
    }

    fn sprite_sheets(&self) -> Vec<&SpriteSheetName> {
        vec![&self.sprite]
    }

    fn name(&self) -> &EntityName {
        &self.name
    }

    fn bounding_box(&self) -> BoundingBox {
        let width: f32 = self.padding.left as f32 + self.padding.right as f32;
        let height: f32 = self.padding.up as f32 + self.padding.down as f32;
        BoundingBox {
            anchor: self.position.clone(),
            size: PhysicalSize::new(
                self.dimensions.width as f32 + width,
                self.dimensions.height as f32 + height,
            ),
        }
    }

    fn entity_type(&self) -> Type {
        Type::Menu
    }
}
impl FlexItem for ProgressBar {
    fn is_dirty(&mut self) -> bool {
        let dirt = self.is_dirty;
        self.is_dirty = false;
        dirt
    }

    fn has_focus(&self) -> bool {
        false
    }

    fn set_focus(&mut self, _focus: bool) {}

    fn set_position(&mut self, position: &Vector<f32>) {
        self.is_dirty = true;
        self.position = position.clone();
    }
}
