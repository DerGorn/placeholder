use winit::event::KeyEvent;

use super::{entity::{Entity, EntityType}, ressource_descriptor::WindowName};

pub struct Scene<T: EntityType> {
    pub target_window: WindowName,
    pub entities: Vec<Box<dyn Entity<T>>>,
}
impl<T: EntityType> Scene<T> {
    pub fn handle_key_input(&mut self, input: &KeyEvent) {
        for entity in self.entities.iter_mut() {
            entity.handle_key_input(input);
        }
    }
}
