use winit::event::KeyEvent;

use super::{entity::Entity, ressource_descriptor::WindowName};

pub struct Scene {
    pub target_window: WindowName,
    pub entities: Vec<Box<dyn Entity>>,
}
impl Scene {
    pub fn handle_key_input(&mut self, input: &KeyEvent) {
        for entity in self.entities.iter_mut() {
            entity.handle_key_input(input);
        }
    }
}
