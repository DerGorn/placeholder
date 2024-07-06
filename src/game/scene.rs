use crate::{
    create_name_struct,
    graphics::{RenderSceneName, ShaderDescriptor},
};
use winit::event::KeyEvent;

use super::{entity::Entity, ressource_descriptor::WindowName, ExternalEvent};

create_name_struct!(SceneName);

#[derive(Debug)]
pub struct Scene<E: ExternalEvent> {
    pub name: SceneName,
    pub shader_descriptor: ShaderDescriptor,
    pub render_scene: RenderSceneName,
    pub target_window: WindowName,
    pub entities: Vec<Box<dyn Entity<E::EntityType, E>>>,
    pub z_index: i32,
}
impl<E: ExternalEvent> Scene<E> {
    pub fn handle_key_input(&mut self, input: &KeyEvent) -> Vec<E> {
        let mut events = vec![];
        for entity in self.entities.iter_mut() {
            events.append(&mut entity.handle_key_input(input));
        }
        events
    }
}
