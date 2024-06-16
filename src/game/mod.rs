use std::{
    thread,
    time::{Duration, Instant},
};

use crate::app::{IndexBuffer, VertexBuffer};

use super::{
    app::{EventManager, WindowManager},
    graphics::{GraphicsProvider, RenderSceneName, UniformBufferName},
};
use log::{info, warn};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::WindowId};

pub use self::{
    bounding_box::BoundingBox,
    camera::CameraDescriptor,
    entity::{Entity, EntityName, EntityType},
    game_event::{ExternalEvent, GameEvent},
    ressource_descriptor::{RessourceDescriptor, SpriteSheetName, WindowName},
    scene::{Scene, SceneName},
    sprite_sheet::{TextureCoordinates, SpritePosition, SpriteSheet, SpriteSheetDimensions},
    velocity_controller::{Direction, VelocityController},
};
use self::camera::Camera;

mod bounding_box;
mod camera;
mod entity;
mod game_event;
mod ressource_descriptor;
mod scene;
mod sprite;
mod sprite_sheet;
mod velocity_controller;

pub type Index = u16;

pub struct Game<E: ExternalEvent> {
    ressources: RessourceDescriptor,
    active_scenes: Vec<Scene<E>>,
    pending_scenes: Vec<Scene<E>>,
    window_ids: Vec<(WindowName, WindowId)>,
    window_sizes: Vec<(WindowId, PhysicalSize<u32>)>,
    sprite_sheets: Vec<(SpriteSheetName, SpriteSheet)>,
    cameras: Vec<(WindowName, Camera, UniformBufferName)>,
    target_fps: u8,
}
impl<E: ExternalEvent> Game<E> {
    pub fn new(
        ressources: RessourceDescriptor,
        inital_scenes: Vec<Scene<E>>,
        target_fps: u8,
    ) -> Self {
        Self {
            ressources,
            pending_scenes: inital_scenes,
            active_scenes: Vec::new(),
            window_ids: Vec::new(),
            window_sizes: Vec::new(),
            sprite_sheets: Vec::new(),
            cameras: Vec::new(),
            target_fps,
        }
    }

    fn activate_scenes(&mut self, window_manager: &mut WindowManager<GameEvent<E>>) {
        let mut needed_windows = Vec::new();
        let mut scenes_to_discard = Vec::new();
        for scene in self.pending_scenes.iter() {
            if self
                .active_scenes
                .iter()
                .find(|s| s.name == scene.name)
                .is_some()
            {
                warn!("Scene {:?} already exists. Discarding it", scene.name);
                scenes_to_discard.push(scene.name.clone());
                continue;
            }
            if let Some((_, id)) = self
                .window_ids
                .iter()
                .find(|(existing_window, _)| scene.target_window == *existing_window)
            {
                self.request_render_scene(id, window_manager, scene)
            } else {
                if !needed_windows.contains(&scene.target_window) {
                    needed_windows.push(scene.target_window.clone());
                }
            }
        }
        for window_name in needed_windows.iter() {
            let window_descriptor = &self
                .ressources
                .get_window(&window_name)
                .expect(&format!("No ressources provided for {:?}", window_name));
            window_manager.send_event(GameEvent::RequestNewWindow(
                window_descriptor.clone(),
                window_name.clone(),
            ));
        }
        self.pending_scenes
            .retain_mut(|s| !scenes_to_discard.contains(&s.name));
    }

    fn request_render_scene(
        &self,
        target_window: &WindowId,
        window_manager: &mut WindowManager<GameEvent<E>>,
        scene: &Scene<E>,
    ) {
        window_manager.send_event(GameEvent::RequestNewRenderScene(
            target_window.clone(),
            scene.render_scene.clone(),
            scene.shader_descriptor.clone(),
            scene.index_format.clone(),
            scene.vertex_buffer_layout.clone(),
        ));
    }

    fn request_sprite_sheet(
        &self,
        name: &SpriteSheetName,
        render_scenes: Vec<RenderSceneName>,
        window_manager: &mut WindowManager<GameEvent<E>>,
    ) {
        let path = &self
            .ressources
            .get_sprite_sheet(&name)
            .expect(&format!(
                "No source path provided for SpriteSheet '{:?}'",
                name
            ))
            .0;
        window_manager.send_event(GameEvent::RequestNewSpriteSheet(
            name.clone(),
            path.clone(),
            render_scenes,
        ));
    }

    fn get_window_name(&self, id: &WindowId) -> Option<&WindowName> {
        self.window_ids
            .iter()
            .find(|(_, i)| i == id)
            .map(|(name, _)| name)
    }

    // fn get_scenes(&self, window_name: &WindowName) -> Vec<&Scene<T>> {
    //     self.active_scenes
    //         .iter()
    //         .filter(|scene| scene.target_window == *window_name)
    //         .collect()
    // }

    fn get_scenes_mut(&mut self, window_name: &WindowName) -> Vec<&mut Scene<E>> {
        self.active_scenes
            .iter_mut()
            .filter(|scene| scene.target_window == *window_name)
            .collect()
    }
}
impl<E: ExternalEvent + 'static> EventManager<GameEvent<E>> for Game<E> {
    fn window_event(
        &mut self,
        _window_manager: &mut WindowManager<GameEvent<E>>,
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
                        if let Some((_, camera, _)) =
                            self.cameras.iter_mut().find(|(n, _, _)| n == &window_name)
                        {
                            camera.handle_key_input(event);
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
        window_manager: &mut WindowManager<GameEvent<E>>,
        graphics_provider: &mut GraphicsProvider,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: GameEvent<E>,
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
                for scene in self
                    .pending_scenes
                    .iter()
                    .filter(|s| s.target_window == name)
                {
                    self.request_render_scene(&id, window_manager, scene);
                }
                if let Some(camera_descriptor) = &self.ressources.get_camera(&name) {
                    let camera: Camera = camera_descriptor.into();
                    graphics_provider.create_uniform_buffer(
                        name.as_str(),
                        &camera.as_bytes(),
                        wgpu::ShaderStages::VERTEX,
                    );
                    self.cameras
                        .push((name.clone(), camera, name.as_str().into()));
                }
            }
            GameEvent::NewRenderScene(render_scene) => {
                let index = self
                    .pending_scenes
                    .iter()
                    .position(|scene| scene.render_scene == render_scene)
                    .expect("Scene Vanished before getting created fully");
                for sprite_sheet in self.pending_scenes[index]
                    .entities
                    .iter()
                    .filter_map(|e| e.sprite_sheet())
                {
                    self.request_sprite_sheet(
                        &sprite_sheet,
                        vec![render_scene.clone()],
                        window_manager,
                    );
                }
                let scene = self.pending_scenes.remove(index);
                window_manager.send_event(GameEvent::External(E::new_scene(&scene)));
                self.active_scenes.push(scene);
                self.active_scenes.sort_by_key(|s| s.z_index);
            }
            GameEvent::NewSpriteSheet(label, None) => {
                panic!("Could not load SpriteSheet '{:?}'", label)
                // self.request_sprite_sheet(label, window_manager)
            }
            GameEvent::NewSpriteSheet(label, Some(id)) => {
                if self
                    .sprite_sheets
                    .iter()
                    .find(|(l, _)| label == *l)
                    .is_none()
                {
                    let dimensions = &self
                        .ressources
                        .get_sprite_sheet(&label)
                        .expect(&format!(
                            "No dimensions provided for SpriteSheet '{:?}'",
                            label
                        ))
                        .1;
                    let sprite_sheet = SpriteSheet::new(id, dimensions);
                    self.sprite_sheets.push((label.clone(), sprite_sheet));
                }
            }
            GameEvent::Timer(delta_t) => {
                for scene in self.active_scenes.iter_mut() {
                    let mut vertices = VertexBuffer::new();
                    let mut indices = IndexBuffer::new();
                    let entities = &mut scene.entities;
                    entities.sort_by(|a, b| a.z().partial_cmp(&b.z()).expect("NaN NaN NaN"));
                    for i in 0..entities.len() {
                        let (left, right) = entities.split_at_mut(i);
                        let (entity, right) = right.split_first_mut().unwrap();
                        let interactions = left.iter().chain(right.iter()).map(|e| &*e).collect();
                        let events = entity.update(&interactions, &delta_t);
                        for event in events {
                            window_manager.send_event(GameEvent::External(event))
                        }
                        let sprite_sheet = if let Some(entity_sprite_sheet) = entity.sprite_sheet()
                        {
                            self.sprite_sheets
                                .iter()
                                .find(|(l, _)| l == entity_sprite_sheet)
                                .map(|(_, s)| s)
                        } else {
                            None
                        };
                        entity.render(&mut vertices, &mut indices, sprite_sheet);
                    }
                    if let Some((_, camera, camera_name)) = self
                        .cameras
                        .iter_mut()
                        .find(|(n, _, _)| n == &scene.target_window)
                    {
                        match camera.update(entities.iter().map(|e| &*e).collect(), &delta_t) {
                            Ok(()) => {}
                            Err(err) => info!("Camera update failed: {}", err),
                        };
                        graphics_provider.update_uniform_buffer(camera_name, &camera.as_bytes());
                    }
                    window_manager.send_event(GameEvent::RenderUpdate(
                        scene.render_scene.clone(),
                        vertices,
                        indices,
                    ));
                }
            }
            GameEvent::External(event) => {
                if event.is_request_new_scenes() {
                    let scenes = event
                        .consume_scenes_request()
                        .expect("Somehow generated no Scene");
                    self.pending_scenes.extend(scenes);
                    self.activate_scenes(window_manager);
                    return;
                }
                println!("EXTERN EVENT: {:?}", event);
            }
            _ => {}
        }
    }
}
