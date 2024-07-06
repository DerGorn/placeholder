use std::{
    thread,
    time::{Duration, Instant},
};

use crate::{
    app::{IndexBuffer, VertexBuffer},
    graphics_provider::ShaderDescriptor,
};

use super::{
    app::{EventManager, WindowManager},
    graphics::{GraphicsProvider, RenderSceneName, UniformBufferName},
};
use log::{info, warn};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::WindowId};

use self::camera::Camera;
pub use self::{
    bounding_box::BoundingBox,
    camera::CameraDescriptor,
    entity::{Entity, EntityName, EntityType},
    game_event::{ExternalEvent, GameEvent},
    ressource_descriptor::{RessourceDescriptor, SpriteSheetName, WindowName},
    scene::{Scene, SceneName},
    sprite_sheet::{SpritePosition, SpriteSheet, SpriteSheetDimensions, TextureCoordinates},
    velocity_controller::{Direction, VelocityController},
};

mod bounding_box;
mod camera;
mod entity;
mod game_event;
mod ressource_descriptor;
mod scene;
mod sprite_sheet;
mod velocity_controller;

pub trait State<E: ExternalEvent> {
    fn handle_event(&mut self, event: E) -> Vec<E>;
}

pub struct Game<E: ExternalEvent, S: State<E>> {
    ressources: RessourceDescriptor,
    active_scenes: Vec<Scene<E>>,
    pending_scenes: Vec<Scene<E>>,
    suspended_scenes: Vec<Scene<E>>,
    window_ids: Vec<(WindowName, WindowId)>,
    window_sizes: Vec<(WindowId, PhysicalSize<u32>)>,
    sprite_sheets: Vec<(SpriteSheetName, SpriteSheet)>,
    cameras: Vec<(SceneName, Camera, UniformBufferName)>,
    target_fps: u8,
    state: S,
}
impl<E: ExternalEvent, S: State<E>> Game<E, S> {
    pub fn new(
        ressources: RessourceDescriptor,
        inital_scenes: Vec<Scene<E>>,
        target_fps: u8,
        state: S,
    ) -> Self {
        Self {
            ressources,
            pending_scenes: inital_scenes,
            active_scenes: Vec::new(),
            suspended_scenes: Vec::new(),
            window_ids: Vec::new(),
            window_sizes: Vec::new(),
            sprite_sheets: Vec::new(),
            cameras: Vec::new(),
            target_fps,
            state,
        }
    }

    fn activate_scenes(&mut self, window_manager: &mut WindowManager<GameEvent<E>>) {
        let mut needed_windows = Vec::new();
        let mut scenes_to_discard = Vec::new();
        let mut scenes_to_request = Vec::new();
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
                scenes_to_request.push((
                    id.clone(),
                    scene.render_scene.clone(),
                    scene.name.clone(),
                    scene.shader_descriptor.clone(),
                ));
            } else {
                if !needed_windows.contains(&scene.target_window) {
                    needed_windows.push(scene.target_window.clone());
                }
            }
        }
        for (window_id, render_scene, scene, shader_descriptor) in scenes_to_request {
            self.request_render_scene(
                &window_id,
                window_manager,
                render_scene,
                scene,
                shader_descriptor,
            );
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
        &mut self,
        target_window: &WindowId,
        window_manager: &mut WindowManager<GameEvent<E>>,
        render_scene: RenderSceneName,
        scene: SceneName,
        shader_descriptor: ShaderDescriptor,
    ) {
        let (camera, render_scene_descriptor) = self.ressources.get_render_scene(&render_scene);
        let mut uniform_buffers: Vec<(UniformBufferName, Vec<u8>, wgpu::ShaderStages)> =
            shader_descriptor
                .uniforms
                .iter()
                .map(|name| {
                    self.ressources
                        .get_uniform(&(*name).into())
                        .expect(&format!(
                            "Did not specify UniformBuffer {:?} in RessourceDescriptor",
                            name
                        ))
                })
                .collect();
        if let Some(camera_descriptor) = camera {
            let camera: Camera = (&camera_descriptor).into();
            let uniform_name = &format!("{:?} camera", render_scene.as_str());
            let bytes = camera.as_bytes();
            self.cameras
                .push((scene.clone(), camera, uniform_name.into()));
            uniform_buffers.push((uniform_name.into(), bytes, wgpu::ShaderStages::VERTEX));
        };
        window_manager.send_event(GameEvent::RequestNewRenderScene(
            target_window.clone(),
            render_scene,
            shader_descriptor,
            render_scene_descriptor,
            uniform_buffers,
        ));
    }

    fn request_sprite_sheet(
        &self,
        name: &SpriteSheetName,
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
        window_manager.send_event(GameEvent::RequestNewSpriteSheet(name.clone(), path.clone()));
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

    // fn get_scenes_mut(&mut self, window_name: &WindowName) -> Vec<&mut Scene<E>> {
    //     self.active_scenes
    //         .iter_mut()
    //         .filter(|scene| scene.target_window == *window_name)
    //         .collect()
    // }
}
impl<E: ExternalEvent + 'static, S: State<E>> EventManager<GameEvent<E>> for Game<E, S> {
    fn window_event(
        &mut self,
        window_manager: &mut WindowManager<GameEvent<E>>,
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
                        for scene in self
                            .active_scenes
                            .iter_mut()
                            .filter(|scene| scene.target_window == window_name)
                        {
                            let events = scene.handle_key_input(event);
                            if let Some((_, camera, _)) =
                                self.cameras.iter_mut().find(|(n, _, _)| n == &scene.name)
                            {
                                camera.handle_key_input(event);
                            }
                            for event in events {
                                window_manager.send_event(GameEvent::External(event));
                            }
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
                for i in 0..self.pending_scenes.len() {
                    let scene = &self.pending_scenes[i];
                    if scene.target_window == name {
                        self.request_render_scene(
                            &id,
                            window_manager,
                            scene.render_scene.clone(),
                            scene.name.clone(),
                            scene.shader_descriptor.clone(),
                        );
                    }
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
                    .map(|e| e.sprite_sheets())
                    .flatten()
                {
                    self.request_sprite_sheet(&sprite_sheet, window_manager);
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
                        let events = entity.update(&interactions, &delta_t, &scene.name);
                        for event in events {
                            window_manager.send_event(GameEvent::External(event))
                        }
                        let sprite_sheets = entity
                            .sprite_sheets()
                            .iter()
                            .map(|entity_sprite_sheet| {
                                self.sprite_sheets
                                    .iter()
                                    .find(|(l, _)| l == *entity_sprite_sheet)
                                    .map(|(_, s)| s)
                            })
                            .collect();
                        entity.render(&mut vertices, &mut indices, sprite_sheets);
                    }
                    if let Some((_, camera, camera_name)) =
                        self.cameras.iter_mut().find(|(n, _, _)| n == &scene.name)
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
                println!("EXTERN EVENT: {:?}", event);
                if event.is_request_new_scenes() {
                    info!("Creating new Scenes");
                    let scenes = event
                        .consume_scenes_request()
                        .expect("Somehow generated no Scene");
                    self.pending_scenes.extend(scenes);
                    self.activate_scenes(window_manager);
                    return;
                }
                if let Some(suspendable_scene) = event.is_request_suspend_scene() {
                    info!("Suspending Scene {:?}", suspendable_scene);
                    if let Some(index) = self
                        .active_scenes
                        .iter()
                        .position(|s| s.name == *suspendable_scene)
                    {
                        let scene = self.active_scenes.remove(index);
                        self.suspended_scenes.push(scene);
                        self.cameras
                            .iter_mut()
                            .filter(|(s, _, _)| s == suspendable_scene)
                            .for_each(|(_, camera, _)| camera.reset_offset());
                    } else {
                        warn!(
                            "Tried to suspend Scene {:?}, but it is not active",
                            suspendable_scene
                        );
                    }
                }
                if let Some(activatable_scene) = event.is_request_activate_suspended_scene() {
                    info!("Activating Scene: {:?}", activatable_scene);
                    if let Some(index) = self
                        .suspended_scenes
                        .iter()
                        .position(|s| s.name == *activatable_scene)
                    {
                        let scene = self.suspended_scenes.remove(index);
                        self.active_scenes.push(scene);
                        self.active_scenes.sort_by_key(|s| s.z_index);
                    } else {
                        warn!(
                            "Tried to activate suspended Scene {:?}, but it is not suspended",
                            activatable_scene
                        );
                    }
                }
                if let Some(deletable_scene) = event.is_request_delete_scene() {
                    info!("Deleting Scene {:?}", deletable_scene);
                    if let Some(active_index) = self
                        .active_scenes
                        .iter()
                        .position(|s| s.name == *deletable_scene)
                    {
                        let scene = self.active_scenes.remove(active_index);
                        graphics_provider.remove_render_scene(&scene.render_scene);
                    } else if let Some(suspended_index) = self
                        .suspended_scenes
                        .iter()
                        .position(|s| s.name == *deletable_scene)
                    {
                        let scene = self.suspended_scenes.remove(suspended_index);
                        graphics_provider.remove_render_scene(&scene.render_scene);
                    } else {
                        warn!(
                            "Tried to delete Scene {:?}, but its neither active nor suspended",
                            deletable_scene
                        );
                    }
                    self.cameras
                        .retain(|(scene_name, _, _)| scene_name != deletable_scene);
                }
                if let Some((uniform_name, contents)) = event.is_update_uniform_buffer() {
                    graphics_provider.update_uniform_buffer(uniform_name, contents);
                }
                if let Some((entity, scene)) = event.is_delete_entity() {
                    info!("Deleting Entiy {:?} from Scene {:?}", entity, scene);
                    let scene = self
                        .active_scenes
                        .iter_mut()
                        .find(|s| s.name == *scene)
                        .unwrap_or_else(|| {
                            self.suspended_scenes
                                .iter_mut()
                                .find(|s| s.name == *scene)
                                .expect(&format!("Found no active nor suspended scene {:?}", scene))
                        });
                    scene.entities.retain(|e| e.name() != entity);
                }
                if event.is_end_game() {
                    window_manager.send_event(GameEvent::EndGame);
                    return;
                }
                let response_events = self.state.handle_event(event);
                for event in response_events {
                    window_manager.send_event(GameEvent::External(event));
                }
            }
            _ => {}
        }
    }
}
