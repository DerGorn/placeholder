use crate::{
    create_name_struct,
    graphics::{RenderSceneName, ShaderDescriptor},
    graphics_provider::RenderSceneDescriptor,
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
    pub render_scene_descriptor: RenderSceneDescriptor,
}
impl<E: ExternalEvent> Scene<E> {
    pub fn handle_key_input(&mut self, input: &KeyEvent) {
        for entity in self.entities.iter_mut() {
            entity.handle_key_input(input);
        }
    }
}
