use std::fmt::Debug;

use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, Entity, EntityName, SpritePosition, SpriteSheet},
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

pub struct Button {
    position: Vector<f32>,
    name: EntityName,
    text: Text,
    is_dirty: bool,
    is_focused: bool,
    low_color: Option<&'static Color>,
    high_color: Option<&'static Color>,
}
impl Button {
    pub fn new(
        text: String,
        name: EntityName,
        size: PhysicalSize<u16>,
        position: Vector<f32>,
        font_size: u8,
        fit_to_content: bool,
    ) -> Self {
        Self {
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
            low_color: Some(&UNFOCUS_LOW_COLOR),
            high_color: Some(&UNFOCUS_HIGH_COLOR),
        }
    }

    pub fn set_focus(&mut self, is_focused: bool) {
        self.is_focused = is_focused;
        if self.is_focused {
            self.text.color = FOCUS_HIGH_COLOR;
            self.low_color = Some(&FOCUS_LOW_COLOR);
            self.high_color = Some(&FOCUS_HIGH_COLOR);
        } else {
            self.text.color = UNFOCUS_HIGH_COLOR;
            self.low_color = Some(&UNFOCUS_LOW_COLOR);
            self.high_color = Some(&UNFOCUS_HIGH_COLOR);
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
        if !sprite_sheet[0].is_none() {
            let sprite_sheet = SpriteSheet::default();
            let bbox = self.bounding_box();
            let border_thickness = 4.0;
            render_ui_sprite(
                &bbox,
                vertices,
                indices,
                &sprite_sheet,
                &SpritePosition::new(0, 0),
                self.low_color,
            );
            render_ui_box_border(
                &bbox,
                vertices,
                indices,
                border_thickness,
                self.high_color.unwrap(),
            );
        }
        self.text.render(vertices, indices, sprite_sheet);
    }
    fn bounding_box(&self) -> BoundingBox {
        self.text.bounding_box()
    }
    fn name(&self) -> &EntityName {
        &self.name
    }
    fn sprite_sheets(&self) -> Vec<&placeholder::game_engine::SpriteSheetName> {
        self.text.sprite_sheets()
    }
    fn entity_type(&self) -> Type {
        Type::Menu
    }
}
impl FlexItem for Button {
    fn position_mut(&mut self) -> &mut Vector<f32> {
        self.is_dirty = true;
        self.text.position_mut()
    }

    fn is_dirty(&mut self) -> bool {
        self.text.is_dirty()
    }
}
