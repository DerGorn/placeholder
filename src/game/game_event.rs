use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use placeholder::{
    app::{ApplicationEvent, WindowDescriptor},
    graphics::{RenderSceneName, ShaderDescriptor},
};
use winit::window::WindowId;

use crate::vertex::Vertex;

use super::{
    ressource_descriptor::{SpriteSheetName, WindowName},
    Index,
};

#[derive(Debug)]
pub enum GameEvent {
    Timer(Duration),
    Resumed,
    NewWindow(WindowId, WindowName),
    RequestNewWindow(WindowDescriptor, WindowName),
    RenderUpdate(RenderSceneName, Vec<Vertex>, Vec<Index>),
    NewSpriteSheet(SpriteSheetName, Option<u32>),
    RequestNewSpriteSheet(SpriteSheetName, PathBuf, Vec<RenderSceneName>),
    NewRenderScene(RenderSceneName),
    RequestNewRenderScene(WindowId, RenderSceneName, ShaderDescriptor),
}
impl ApplicationEvent<Index, Vertex> for GameEvent {
    fn app_resumed() -> Self {
        Self::Resumed
    }

    fn is_request_new_window<'a>(&'a self) -> Option<(&'a WindowDescriptor, &'a str)> {
        if let Self::RequestNewWindow(window_descriptor, name) = self {
            Some((&window_descriptor, name.as_str()))
        } else {
            None
        }
    }

    fn is_render_update<'a>(
        &'a self,
    ) -> Option<(
        &'a RenderSceneName,
        Option<&'a [Index]>,
        Option<&'a [Vertex]>,
    )> {
        if let Self::RenderUpdate(render_scene, vertices, indices) = self {
            Some((
                &render_scene,
                if vertices.len() > 0 {
                    Some(indices.as_slice())
                } else {
                    None
                },
                if indices.len() > 0 {
                    Some(vertices.as_slice())
                } else {
                    None
                },
            ))
        } else {
            None
        }
    }

    fn is_request_new_texture<'a>(&'a self) -> Option<(&'a Path, &'a str, &[RenderSceneName])> {
        if let Self::RequestNewSpriteSheet(label, path, render_scenes) = self {
            Some((path, label.as_str(), render_scenes.as_slice()))
        } else {
            None
        }
    }

    fn is_request_new_render_scene<'a>(
        &'a self,
    ) -> Option<(&'a WindowId, &'a RenderSceneName, &'a ShaderDescriptor)> {
        if let Self::RequestNewRenderScene(window_id, render_scene, shader_descriptor) = self {
            Some((window_id, render_scene, shader_descriptor))
        } else {
            None
        }
    }

    fn new_render_scene(render_scene: &RenderSceneName) -> Self {
        GameEvent::NewRenderScene(render_scene.clone())
    }

    fn new_texture(label: &str, id: Option<u32>) -> Self {
        Self::NewSpriteSheet(label.into(), id)
    }

    fn new_window(id: &WindowId, name: &str) -> Self {
        Self::NewWindow(id.clone(), name.into())
    }
}
