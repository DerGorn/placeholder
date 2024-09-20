use std::fmt::Debug;

use ferride_core::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, Entity, EntityName, SpritePosition, SpriteSheet, SpriteSheetName},
    graphics::DEFAULT_TEXTURE,
    reexports::winit::{keyboard::PhysicalKey, PhysicalSize},
};
use threed::Vector;

use crate::{
    color::Color,
    ui::{FlexItem, Text},
    vertex::{render_ui_box_border, render_ui_sprite},
    Event, Type,
};

const FOCUS_HIGH_COLOR: Color = Color::new_rgba(255, 0, 0, 255);
const FOCUS_LOW_COLOR: Color = Color::new_rgba(82, 5, 5, 160);
const UNFOCUS_HIGH_COLOR: Color = Color::new_rgba(24, 25, 27, 255);
pub const UNFOCUS_LOW_COLOR: Color = Color::new_rgba(0, 0, 0, 160);
const HIGHLIGHT_HIGH_COLOR: Color = Color::new_rgba(255, 255, 0, 255);
const HIGHLIGHT_LOW_COLOR: Color = Color::new_rgba(82, 82, 5, 160);

pub struct ColorPair {
    pub high: Color,
    pub low: Color,
}
impl ColorPair {
    pub fn new(high: Color, low: Color) -> Self {
        Self { high, low }
    }
}
pub struct BorderBoxStyle {
    pub focus: ColorPair,
    pub unfocus: ColorPair,
    pub highlight: ColorPair,
}
pub struct ImageStyle {
    pub sprite_sheet: SpriteSheetName,
    pub focus_sprite: SpritePosition,
    pub unfocus_sprite: SpritePosition,
    pub highlight_sprite: SpritePosition,
}
pub struct BackgroundImageStyle {
    pub focus: Color,
    pub unfocus: Color,
    pub highlight: Color,
    pub sprite_sheet: SpriteSheetName,
    pub focus_sprite: SpritePosition,
    pub unfocus_sprite: SpritePosition,
    pub highlight_sprite: SpritePosition,
    pub with_border: bool,
}
impl Default for BackgroundImageStyle {
    fn default() -> Self {
        BackgroundImageStyle {
            focus: FOCUS_HIGH_COLOR,
            unfocus: UNFOCUS_HIGH_COLOR,
            highlight: HIGHLIGHT_HIGH_COLOR,
            sprite_sheet: DEFAULT_TEXTURE.into(),
            focus_sprite: SpritePosition::new(0, 0),
            unfocus_sprite: SpritePosition::new(0, 0),
            highlight_sprite: SpritePosition::new(0, 0),
            with_border: true,
        }
    }
}
pub struct PlainStyle {
    pub focus: Color,
    pub unfocus: Color,
    pub highlight: Color,
}
pub struct UnderLineStyle {
    pub focus: Color,
    pub unfocus: Color,
    pub highlight: Color,
}

#[allow(dead_code)]
pub enum ButtonStyle {
    /// BorderBox(FOCUS_HIGH_COLOR, FOCUS_LOW_COLOR, UNFOCUS_HIGH_COLOR, UNFOCUS_LOW_COLOR)
    BorderBox(BorderBoxStyle),
    /// Image(SpriteSheet, FOCUS_SPRITE, UNFOCUS_SPRITE)
    Image(ImageStyle),
    /// Image(FOCUS_COLOR, UNFOCUS_COLOR, SpriteSheet, FOCUS_SPRITE, UNFOCUS_SPRITE)
    BackgroundImage(BackgroundImageStyle),
    /// Plain(FOCUS_COLOR, UNFOCUS_COLOR)
    Plain(PlainStyle),
    /// UnderLine(FOCUS_COLOR, UNFOCUS_COLOR)
    UnderLine(UnderLineStyle),
}
impl Default for ButtonStyle {
    fn default() -> Self {
        Self::BorderBox(BorderBoxStyle {
            focus: ColorPair::new(FOCUS_HIGH_COLOR, FOCUS_LOW_COLOR),
            unfocus: ColorPair::new(UNFOCUS_HIGH_COLOR, UNFOCUS_LOW_COLOR),
            highlight: ColorPair::new(HIGHLIGHT_HIGH_COLOR, HIGHLIGHT_LOW_COLOR),
        })
    }
}

const BORDER_THICKNESS: f32 = 4.0;

pub struct Button {
    position: Vector<f32>,
    name: EntityName,
    text: Text,
    is_dirty: bool,
    is_focused: bool,
    is_highlighted: bool,
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
            is_highlighted: false,
            style,
        };
        button.set_focus(false);
        button
    }
}
impl Button {
    pub fn set_highlighted(&mut self, is_highlighted: bool) {
        if self.is_highlighted == is_highlighted {
            return;
        }
        self.is_highlighted = is_highlighted;
        self.update_text_color()
    }

    pub fn set_content(&mut self, text: String) {
        self.text.set_text(text);
        self.is_dirty = true;
    }

    fn update_text_color(&mut self) {
        let colors = match &self.style {
            ButtonStyle::BorderBox(style) => Some((
                style.focus.high.clone(),
                style.unfocus.high.clone(),
                style.highlight.high.clone(),
            )),
            ButtonStyle::Plain(style) => Some((
                style.focus.clone(),
                style.unfocus.clone(),
                style.highlight.clone(),
            )),
            ButtonStyle::UnderLine(style) => Some((
                style.focus.clone(),
                style.unfocus.clone(),
                style.highlight.clone(),
            )),
            ButtonStyle::BackgroundImage(style) => Some((
                style.focus.clone(),
                style.unfocus.clone(),
                style.highlight.clone(),
            )),
            ButtonStyle::Image(_) => None,
        };
        if let Some((focus, unfocus, highlight)) = colors {
            if self.is_focused {
                self.text.color = focus;
            } else if self.is_highlighted {
                self.text.color = highlight;
            } else {
                self.text.color = unfocus;
            }
        }
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
    fn handle_key_input(&mut self, input: &ferride_core::reexports::winit::event::KeyEvent) -> Vec<Event> {
        if self.is_focused && input.state == ferride_core::reexports::winit::event::ElementState::Pressed {
            match input.physical_key {
                PhysicalKey::Code(key_code) => {
                    vec![Event::ButtonPressed(self.name.clone(), key_code)]
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
        scene: &ferride_core::game_engine::SceneName,
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
            ButtonStyle::BorderBox(style) => {
                let low_color = Some(if self.is_focused {
                    &style.focus.low
                } else if self.is_highlighted {
                    &style.highlight.low
                } else {
                    &style.unfocus.low
                });
                let high_color = Some(if self.is_focused {
                    &style.focus.high
                } else if self.is_highlighted {
                    &style.highlight.high
                } else {
                    &style.unfocus.high
                });
                if !sprite_sheet[0].is_none() {
                    let sprite_sheet = SpriteSheet::default();
                    let mut bbox = self.bounding_box();
                    bbox.size.height -= 2.0 * BORDER_THICKNESS;
                    bbox.size.width -= 2.0 * BORDER_THICKNESS;
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
                        high_color.expect("No high color"),
                    );
                }
            }
            ButtonStyle::Plain(_) => {}
            ButtonStyle::UnderLine(UnderLineStyle { focus, .. }) => {
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
            ButtonStyle::Image(_) => {
                todo!("implement ButtonStyle::Image");
            }
            ButtonStyle::BackgroundImage(style) => {
                if let Some(sprite_sheet) = sprite_sheet[1] {
                    let sprite_position = if self.is_highlighted {
                        &style.highlight_sprite
                    } else if self.is_focused {
                        &style.focus_sprite
                    } else {
                        &style.unfocus_sprite
                    };
                    let border_color = Some(if self.is_focused {
                        &style.focus
                    } else if self.is_highlighted {
                        &style.highlight
                    } else {
                        &style.unfocus
                    });
                    let mut bbox = self.bounding_box();
                    bbox.size.height -= 2.0 * BORDER_THICKNESS;
                    bbox.size.width -= 2.0 * BORDER_THICKNESS;
                    render_ui_sprite(
                        &bbox,
                        vertices,
                        indices,
                        sprite_sheet,
                        sprite_position,
                        None,
                    );
                    render_ui_box_border(
                        &bbox,
                        vertices,
                        indices,
                        BORDER_THICKNESS,
                        border_color.expect("No border color"),
                    );
                }
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
        let mut resize_bbox = || {
            bbox.size.height += 2.0 * BORDER_THICKNESS;
            bbox.size.width += 2.0 * BORDER_THICKNESS;
            bbox.anchor += Vector::new(BORDER_THICKNESS, BORDER_THICKNESS, 0.0);
        };
        match self.style {
            ButtonStyle::BorderBox(_) => {
                resize_bbox();
            }
            ButtonStyle::BackgroundImage(BackgroundImageStyle { with_border, .. })
                if with_border =>
            {
                resize_bbox();
            }
            _ => {}
        }
        bbox
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn sprite_sheets(&self) -> Vec<&ferride_core::game_engine::SpriteSheetName> {
        let mut sprite_sheets = self.text.sprite_sheets();
        match &self.style {
            ButtonStyle::Image(ImageStyle { sprite_sheet, .. })
            | ButtonStyle::BackgroundImage(BackgroundImageStyle { sprite_sheet, .. }) => {
                sprite_sheets.push(sprite_sheet);
            }
            _ => {}
        };
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
        if self.is_focused == is_focused {
            return;
        }
        self.is_focused = is_focused;
        self.update_text_color();
    }

    fn has_focus(&self) -> bool {
        self.is_focused
    }
}
