use log::warn;
use placeholder::game_engine::{BoundingBox, Entity, EntityName, SpritePosition, SpriteSheetName};
use std::fmt::Debug;
use threed::Vector;
use winit::{dpi::PhysicalSize, keyboard::PhysicalKey};

use crate::{
    character::ui::CharacterGui, event::Event, impl_flex_struct, ui::{Alignment, FlexDirection, FlexItem, FlexOrigin}, vertex::render_ui_sprite, Type
};

mod button;
pub use button::{Button, ButtonStyle};

use super::ProgressBar;
pub mod button_styles {
    pub use super::button::{
        BackgroundImageStyle, BorderBoxStyle, ColorPair, ImageStyle, PlainStyle, UnderLineStyle, UNFOCUS_LOW_COLOR,
    };
}

macro_rules! impl_flex_button_manager {
    ($name: ident, $child_type: ty, up: $($up_key:ident),*; down: $($down_key:ident),*) => {
        pub type $name = FlexInputManager<$child_type>;
        impl $name {
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
                has_focus: bool,
                mut children: Vec<Box<$child_type>>,
            ) -> Self {
                let number_of_sprites = children.iter().map(|x| x.sprite_sheets().len()).collect();
                let mut focused_child = None;
                if has_focus && children.len() > 0 {
                    let index = children.iter().position(|c| c.has_focus()).unwrap_or(0);
                    for i in 0..children.len() {
                        children[i].set_focus(i == index);
                    }
                    focused_child = Some(index);
                }
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
                    focused_child,
                    has_focus,
                    down_keys: vec![
                        $(
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::$down_key),
                        )*
                    ],
                    up_keys: vec![
                        $(
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::$up_key),
                        )*
                    ],
                }
            }
        }
    };
}
impl_flex_button_manager!(FlexButtonLineManager, FlexButtonLine, up: KeyE; down: KeyQ);
impl_flex_button_manager!(FlexButtonLine, Button, up: KeyW, KeyA; down: KeyS, KeyD);
impl_flex_button_manager!(FlexProgressBarLine, ProgressBar, up: ; down:);
impl_flex_button_manager!(FlexCharacterGuiLine, CharacterGui, up: KeyW, KeyA; down: KeyS, KeyD);
impl_flex_button_manager!(FlexCharacterGuiLineManager, FlexCharacterGuiLine, up: KeyE; down: KeyQ);

pub struct FlexInputManager<T: FlexItem> {
    flex_direction: FlexDirection,
    flex_origin: FlexOrigin,
    /// Alignment of children orthogonal to the flex direction
    align_content: Alignment,
    background_image: Option<(SpriteSheetName, SpritePosition)>,
    gap: f32,
    dimensions: PhysicalSize<u16>,
    position: Vector<f32>,
    pub children: Vec<Box<T>>,
    name: EntityName,
    shrink_to_content: bool,
    number_of_sprites: Vec<usize>,
    is_dirty: bool,
    focused_child: Option<usize>,
    has_focus: bool,
    /// W or A
    up_keys: Vec<PhysicalKey>,
    /// S or D
    down_keys: Vec<PhysicalKey>,
}
impl<T: FlexItem> FlexInputManager<T> {
    pub fn focus_child(&mut self, index: usize) {
        if index >= self.children.len() {
            warn!(
                "{:?}: Trying to focus non existing child {}",
                self.name, index
            );
        }
        if let Some(focused_child) = self.focused_child {
            self.children[focused_child].set_focus(false);
        }
        self.focused_child = Some(index);
        self.children[index].set_focus(true);
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
impl_flex_struct!(FlexInputManager<T: FlexItem>);
impl<T: FlexItem> Debug for FlexInputManager<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("FlexInputManager<{:?}>", stringify!(T)))
            .field("name", &self.name)
            .field("position", &self.position)
            .finish()
    }
}
impl<T: FlexItem> Entity<Type, Event> for FlexInputManager<T> {
    fn delete_child_entity(&mut self, name: &EntityName) {
        let original_len = self.children.len();
        self.children.retain(|child| child.name() != name);
        for child in &mut self.children {
            child.delete_child_entity(name);
        }
        self.number_of_sprites = self
            .children
            .iter()
            .map(|x| x.sprite_sheets().len())
            .collect();
        self.flex();
        if original_len == self.children.len() {
            return;
        }
        match self.focused_child {
            Some(index) => {
                self.focused_child = Some(index.min(self.children.len() - 1));
                if self.has_focus {
                    self.focus_child(self.focused_child.expect(&format!("{:?}.delete_child_entity with self.focused_child == None and self.active == true", self.name).as_str()));
                }
            }
            None => {}
        }
    }
    fn handle_key_input(&mut self, input: &winit::event::KeyEvent) -> Vec<Event> {
        if !self.has_focus {
            return vec![];
        }

        if input.state == winit::event::ElementState::Pressed {
            let selection_change = match input.physical_key {
                x if self.up_keys.contains(&x) => -1,
                x if self.down_keys.contains(&x) => 1,
                _ => 0,
            };
            let new_focus = (self.focused_child.expect(&format!(
                "{:?}.handle_key_input with self.focused_child == None and self.active == true",
                self.name
            )) as i32
                + selection_change
                + self.children.len() as i32)
                % self.children.len() as i32;
            self.focus_child(new_focus as usize);
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
impl<T: FlexItem> FlexItem for FlexInputManager<T> {
    fn set_position(&mut self, position: &Vector<f32>) {
        // if self.position != *position {
        //     self.is_dirty = true
        // }
        self.flex_set_position(position);
    }

    fn is_dirty(&mut self) -> bool {
        let dirt = self.is_dirty;
        self.is_dirty = false;
        dirt
    }

    fn set_focus(&mut self, focus: bool) {
        if focus == self.has_focus {
            return;
        }
        let focused_child = if let Some(fc) = self.focused_child {
            fc
        } else {
            self.focused_child = Some(0);
            0
        };
        self.children[focused_child].set_focus(focus);
        self.has_focus = focus;
    }

    fn has_focus(&self) -> bool {
        self.has_focus
    }
}
