use std::fmt::Debug;

use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, Entity, EntityName, SpritePosition, SpriteSheet, SpriteSheetName},
};
use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    color::Color,
    ui::{FlexItem, Text},
    vertex::{render_ui_box_border, render_ui_sprite},
    Event, Type,
};

const FOCUS_HIGH_COLOR: Color = Color::new_rgba(255, 0, 0, 255);
const FOCUS_LOW_COLOR: Color = Color::new_rgba(82, 5, 5, 160);
const UNFOCUS_HIGH_COLOR: Color = Color::new_rgba(24, 25, 27, 255);
const UNFOCUS_LOW_COLOR: Color = Color::new_rgba(0, 0, 0, 160);

#[allow(dead_code)]
pub enum ButtonStyle {
    /// BorderBox(FOCUS_HIGH_COLOR, FOCUS_LOW_COLOR, UNFOCUS_HIGH_COLOR, UNFOCUS_LOW_COLOR)
    BorderBox(Color, Color, Color, Color),
    /// Image(SpriteSheet, FOCUS_SPRITE, UNFOCUS_SPRITE)
    Image(SpriteSheetName, SpritePosition, SpritePosition),
    /// Image(FOCUS_COLOR, UNFOCUS_COLOR, SpriteSheet, FOCUS_SPRITE, UNFOCUS_SPRITE)
    BackgroundImage(
        Color,
        Color,
        SpriteSheetName,
        SpritePosition,
        SpritePosition,
    ),
    /// Plain(FOCUS_COLOR, UNFOCUS_COLOR)
    Plain(Color, Color),
    /// UnderLine(FOCUS_COLOR, UNFOCUS_COLOR)
    UnderLine(Color, Color),
}
impl Default for ButtonStyle {
    fn default() -> Self {
        Self::BorderBox(
            FOCUS_HIGH_COLOR,
            FOCUS_LOW_COLOR,
            UNFOCUS_HIGH_COLOR,
            UNFOCUS_LOW_COLOR,
        )
    }
}

const BORDER_THICKNESS: f32 = 4.0;

pub struct Button {
    position: Vector<f32>,
    name: EntityName,
    text: Text,
    is_dirty: bool,
    is_focused: bool,
    style: ButtonStyle,
}
impl Button {
    pub fn new(
        text: String,
        name: EntityName,
        size: PhysicalSize<u16>,
        position: Vector<f32>,
        font_size: crate::ui::text::FontSize,
        fit_to_content: bool,
        style: ButtonStyle,
    ) -> Self {
        let mut button = Self {
            position: position.clone(),
            name: name.clone(),
            text: Text::new(
                text,
                UNFOCUS_HIGH_COLOR,
                name.clone(),
                size,
                position,
                font_size,
                fit_to_content,
            ),
            is_dirty: true,
            is_focused: false,
            style,
        };
        button.set_focus(false);
        button
    }
}
impl Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("name", &self.name)
            .field("text", &self.text)
            .finish()
    }
}
impl Entity<Type, Event> for Button {
    fn handle_key_input(&mut self, input: &winit::event::KeyEvent) -> Vec<Event> {
        if self.is_focused && input.state == winit::event::ElementState::Pressed {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::Space) | PhysicalKey::Code(KeyCode::Enter) => {
                    vec![Event::ButtonPressed(self.name.clone())]
                }
                _ => vec![],
            }
        } else {
            vec![]
        }
    }
    fn update(
        &mut self,
        entities: &Vec<&Box<dyn Entity<Type, Event>>>,
        delta_t: &std::time::Duration,
        scene: &placeholder::game_engine::SceneName,
    ) -> Vec<Event> {
        if self.is_dirty {
            self.position = self.text.position().clone();
            self.is_dirty = false;
        }
        self.text.update(entities, delta_t, scene)
    }
    fn render(
        &mut self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        sprite_sheet: Vec<Option<&SpriteSheet>>,
    ) {
        match &self.style {
            ButtonStyle::BorderBox(focus_high, focus_low, unfocus_high, unfocus_low) => {
                let low_color = Some(if self.is_focused {
                    focus_low
                } else {
                    unfocus_low
                });
                let high_color = Some(if self.is_focused {
                    focus_high
                } else {
                    unfocus_high
                });
                if !sprite_sheet[0].is_none() {
                    let sprite_sheet = SpriteSheet::default();
                    let mut bbox = self.bounding_box();
                    bbox.anchor += Vector::new(BORDER_THICKNESS, BORDER_THICKNESS, 0.0);
                    render_ui_sprite(
                        &bbox,
                        vertices,
                        indices,
                        &sprite_sheet,
                        &SpritePosition::new(0, 0),
                        low_color,
                    );
                    render_ui_box_border(
                        &bbox,
                        vertices,
                        indices,
                        BORDER_THICKNESS,
                        high_color.unwrap(),
                    );
                }
            }
            ButtonStyle::Plain(_, _) => {}
            ButtonStyle::UnderLine(focus, _) => {
                if self.is_focused {
                    let color = Some(focus);
                    if !sprite_sheet[0].is_none() {
                        let sprite_sheet = SpriteSheet::default();
                        let mut bbox = self.bounding_box();
                        bbox.anchor.y -= bbox.size.height / 2.0;
                        bbox.size.height = BORDER_THICKNESS;
                        render_ui_sprite(
                            &bbox,
                            vertices,
                            indices,
                            &sprite_sheet,
                            &SpritePosition::new(0, 0),
                            color,
                        );
                    }
                }
            }
            ButtonStyle::Image(_, _, _) => {
                todo!("implement ButtonStyle::Image");
            }
            ButtonStyle::BackgroundImage(_, _, _, _, _) => {
                todo!("implement ButtonStyle::BackgroundImage");
            }
        }
        let pos = self.text.position();
        let shifted_pos = Vector::new(pos.x - BORDER_THICKNESS, pos.y - BORDER_THICKNESS, 0.0);
        self.text.set_position(&shifted_pos);
        self.text.render(vertices, indices, sprite_sheet);
        self.text.set_position(&pos);
    }
    fn bounding_box(&self) -> BoundingBox {
        let mut bbox = self.text.bounding_box();
        if let ButtonStyle::BorderBox(_, _, _, _) = self.style {
            bbox.size.height += 2.0 * BORDER_THICKNESS;
            bbox.size.width += 2.0 * BORDER_THICKNESS;
            bbox.anchor -= Vector::new(BORDER_THICKNESS, BORDER_THICKNESS, 0.0);
        }
        bbox
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn sprite_sheets(&self) -> Vec<&placeholder::game_engine::SpriteSheetName> {
        let mut sprite_sheets = match &self.style {
            ButtonStyle::Image(sprite_sheet, _, _)
            | ButtonStyle::BackgroundImage(_, _, sprite_sheet, _, _) => {
                vec![sprite_sheet]
            }
            _ => vec![],
        };
        sprite_sheets.extend_from_slice(&mut self.text.sprite_sheets());
        sprite_sheets
    }
    fn entity_type(&self) -> Type {
        Type::Menu
    }
}
impl FlexItem for Button {
    fn set_position(&mut self, position: &Vector<f32>) {
        self.is_dirty = true;
        self.text.set_position(position);
    }

    fn is_dirty(&mut self) -> bool {
        self.text.is_dirty()
    }

    fn set_focus(&mut self, is_focused: bool) {
        self.is_focused = is_focused;
        match &self.style {
            ButtonStyle::BorderBox(focus_high, _, unfocus_high, _) => {
                if is_focused {
                    self.text.color = focus_high.clone();
                } else {
                    self.text.color = unfocus_high.clone();
                }
            }
            ButtonStyle::Plain(focus, unfocus) => {
                if is_focused {
                    self.text.color = focus.clone();
                } else {
                    self.text.color = unfocus.clone();
                }
            }
            ButtonStyle::UnderLine(focus, unfocus) => {
                if is_focused {
                    self.text.color = focus.clone();
                } else {
                    self.text.color = unfocus.clone();
                }
            }
            ButtonStyle::BackgroundImage(focus, unfocus, _, _, _) => {
                if is_focused {
                    self.text.color = focus.clone();
                } else {
                    self.text.color = unfocus.clone();
                }
            }
            ButtonStyle::Image(_, _, _) => {}
        }
    }

    fn has_focus(&self) -> bool {
        self.is_focused
    }
}
