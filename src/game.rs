use std::{
    thread,
    time::{Duration, Instant},
};

use log::warn;
use placeholder::app::{EventManager, WindowManager};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::WindowId};

use self::game_event::GameEvent;
pub use self::{
    entity::Entity,
    ressource_descriptor::{RessourceDescriptor, SpriteSheetName, WindowName},
    scene::Scene,
    sprite::{SpriteDescriptor, SpritePosition},
    sprite_sheet::{SpriteSheet, SpriteSheetDimensions},
};

mod sprite_sheet;

mod sprite;

mod ressource_descriptor;

mod scene;

mod game_event;

mod entity;

pub type Index = u16;

pub struct Game {
    ressources: RessourceDescriptor,
    active_scenes: Vec<Scene>,
    pending_scenes: Vec<Scene>,
    window_ids: Vec<(WindowName, WindowId)>,
    window_sizes: Vec<(WindowId, PhysicalSize<u32>)>,
    sprite_sheets: Vec<(SpriteSheetName, SpriteSheet)>,
    target_fps: u8,
}
impl Game {
    pub fn new(ressources: RessourceDescriptor, inital_scenes: Vec<Scene>, target_fps: u8) -> Self {
        Self {
            ressources,
            pending_scenes: inital_scenes,
            active_scenes: Vec::new(),
            window_ids: Vec::new(),
            window_sizes: Vec::new(),
            sprite_sheets: Vec::new(),
            target_fps,
        }
    }

    fn activate_scenes(&mut self, window_manager: &mut WindowManager<GameEvent>) {
        let mut needed_windows = Vec::new();
        for window_name in self.pending_scenes.iter().map(|scene| &scene.target_window) {
            if self
                .window_ids
                .iter()
                .find(|(existing_window, _)| window_name == existing_window)
                .is_none()
                && !needed_windows.contains(window_name)
            {
                needed_windows.push(window_name.clone());
            }
        }
        for window_name in needed_windows {
            let (window_descriptor, shader_descriptor) = &self
                .ressources
                .get_window(&window_name)
                .expect(&format!("No ressources provided for {:?}", window_name));
            window_manager.send_event(GameEvent::RequestNewWindow(
                window_descriptor.clone(),
                shader_descriptor.clone(),
                window_name.clone(),
            ));
        }
        self.active_scenes.append(&mut self.pending_scenes);
    }

    fn request_sprite_sheet(
        &self,
        name: &SpriteSheetName,
        window_manager: &mut WindowManager<GameEvent>,
    ) {
        let path = &self
            .ressources
            .get_sprite_sheet(&name)
            .expect(&format!(
                "No source path provided for SpriteSheet '{:?}'",
                name
            ))
            .0;
        window_manager.send_event(GameEvent::RequestNewSpriteSheet(name.clone(), path.clone()));
    }

    fn get_window_name(&self, id: &WindowId) -> Option<&WindowName> {
        self.window_ids
            .iter()
            .find(|(_, i)| i == id)
            .map(|(name, _)| name)
    }

    fn get_scenes(&self, window_name: &WindowName) -> Vec<&Scene> {
        self.active_scenes
            .iter()
            .filter(|scene| scene.target_window == *window_name)
            .collect()
    }

    fn get_scenes_mut(&mut self, window_name: &WindowName) -> Vec<&mut Scene> {
        self.active_scenes
            .iter_mut()
            .filter(|scene| scene.target_window == *window_name)
            .collect()
    }
}
impl EventManager<GameEvent> for Game {
    fn window_event(
        &mut self,
        _window_manager: &mut WindowManager<GameEvent>,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        id: &winit::window::WindowId,
        event: &winit::event::WindowEvent,
    ) -> bool
    where
        Self: Sized,
    {
        match event {
            WindowEvent::Resized(size) => {
                let window_size = self.window_sizes.iter_mut().find(|(i, _)| i == id);
                if let Some((_, s)) = window_size {
                    *s = *size
                } else {
                    self.window_sizes.push((id.clone(), *size));
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                match self.get_window_name(id) {
                    Some(window_name) => {
                        let window_name = window_name.clone();
                        for scene in self.get_scenes_mut(&window_name) {
                            scene.handle_key_input(event);
                        }
                    }
                    None => {
                        warn!("No window name found for window id {:?}", id)
                    }
                };
            }
            _ => {}
        }
        true
    }

    fn user_event(
        &mut self,
        window_manager: &mut WindowManager<GameEvent>,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: &GameEvent,
    ) where
        Self: Sized,
    {
        match event {
            GameEvent::Resumed => {
                self.activate_scenes(window_manager);

                let ns_per_frame = 1e9 / (self.target_fps as f64);
                let frame_duration = Duration::from_nanos(ns_per_frame as u64);
                let timer_event_loop = window_manager.create_event_loop_proxy();
                thread::spawn(move || {
                    let mut last_update = Instant::now();
                    loop {
                        match timer_event_loop.send_event(GameEvent::Timer(last_update.elapsed())) {
                            Ok(()) => {}
                            Err(_) => break,
                        };
                        last_update = Instant::now();
                        thread::sleep(frame_duration);
                    }
                });
            }
            GameEvent::NewWindow(id, name) => {
                self.window_ids.push((name.clone(), id.clone()));
                let mut needed_sprite_sheets = Vec::new();
                for scene in self.get_scenes(name) {
                    for entity in &scene.entities {
                        let sprite_sheet = entity.sprite_sheet();
                        if self
                            .sprite_sheets
                            .iter()
                            .find(|(name, _)| name == sprite_sheet)
                            .is_none()
                            && !needed_sprite_sheets.contains(sprite_sheet)
                        {
                            needed_sprite_sheets.push(sprite_sheet.clone())
                        }
                    }
                }
                for sprite_sheet in needed_sprite_sheets {
                    self.request_sprite_sheet(&sprite_sheet, window_manager);
                }
            }
            GameEvent::NewSpriteSheet(label, None) => {
                self.request_sprite_sheet(label, window_manager)
            }
            GameEvent::NewSpriteSheet(label, Some(id)) => {
                let dimensions = &self
                    .ressources
                    .get_sprite_sheet(label)
                    .expect(&format!(
                        "No dimensions provided for SpriteSheet '{:?}'",
                        label
                    ))
                    .1;
                let sprite_sheet = SpriteSheet::new(*id, dimensions);
                self.sprite_sheets.push((label.clone(), sprite_sheet));
            }
            GameEvent::Timer(_delta_t) => {
                for (name, id) in &self.window_ids {
                    let size = self
                        .window_sizes
                        .iter()
                        .find(|(i, _)| i == id)
                        .map(|(_, s)| *s)
                        //Default size if no resize event happened until now
                        .or(Some(PhysicalSize::new(1, 1)))
                        .expect("The universe killed the default");
                    let mut vertices = Vec::new();
                    let mut indices = Vec::new();
                    let mut entities = self
                        .active_scenes
                        .iter_mut()
                        .filter(|scene| scene.target_window == *name)
                        .fold(Vec::new(), |mut entities, scene| {
                            for entity in scene.entities.iter_mut() {
                                entities.push(entity);
                            }
                            entities
                        });
                    entities.sort_by(|a, b| a.z().partial_cmp(&b.z()).expect("NaN NaN NaN"));
                    for entity in entities.iter_mut() {
                        entity.update();
                        let entity_sprite_sheet = entity.sprite_sheet();
                        if let Some((_, sprite_sheet)) = &self
                            .sprite_sheets
                            .iter()
                            .find(|(l, _)| l == entity_sprite_sheet)
                        {
                            entity.render(&mut vertices, &mut indices, &size, sprite_sheet);
                        }
                    }
                    window_manager.send_event(GameEvent::RenderUpdate(*id, vertices, indices));
                }
            }
            _ => {}
        }
    }
}
