use std::path::PathBuf;

use log::info;

use crate::app::WindowDescriptor;
use crate::create_name_struct;

use crate::game_engine::CameraDescriptor;
use crate::graphics_provider::{RenderSceneDescriptor, RenderSceneName, UniformBufferName};

use super::sprite_sheet::SpriteSheetDimensions;

pub struct RessourceDescriptor {
    pub windows: Vec<(WindowName, WindowDescriptor)>,
    /// Per default, a SpriteSheetName n not found in the list will be interpreted as (n,
    /// self.image_directory + n + ".png", (1, 1))
    pub image_directory: PathBuf,
    pub sprite_sheets: Vec<(SpriteSheetName, PathBuf, SpriteSheetDimensions)>,
    ///describes UniformBuffers that are not Cameras, because of their elevated
    pub uniforms: Vec<(UniformBufferName, Vec<u8>, wgpu::ShaderStages)>,
    pub default_render_scene: (Option<CameraDescriptor>, RenderSceneDescriptor),
    pub render_scenes: Vec<(
        Vec<RenderSceneName>,
        Option<CameraDescriptor>,
        RenderSceneDescriptor,
    )>,
}
impl RessourceDescriptor {
    pub fn get_window(&self, name: &WindowName) -> Option<WindowDescriptor> {
        self.windows
            .iter()
            .find(|(window_name, _)| window_name == name)
            .map(|(_, window)| window.clone())
    }
    pub fn get_uniform(
        &self,
        name: &UniformBufferName,
    ) -> Option<(UniformBufferName, Vec<u8>, wgpu::ShaderStages)> {
        self.uniforms
            .iter()
            .find(|(uniform_name, _, _)| uniform_name == name)
            .cloned()
    }
    pub fn get_render_scene(
        &self,
        name: &RenderSceneName,
    ) -> (Option<CameraDescriptor>, RenderSceneDescriptor) {
        let rs = self
            .render_scenes
            .iter()
            .find(|(render_scenes, _, _)| render_scenes.contains(name))
            .map(|(_, camera, descriptor)| (camera.clone(), descriptor.clone()));
        if let Some(render_scene) = rs {
            render_scene
        } else {
            info!("RenderScene {:?} not found. Using default...", name);
            self.default_render_scene.clone()
        }
    }
    pub fn get_sprite_sheet(&self, name: &SpriteSheetName) -> (PathBuf, SpriteSheetDimensions) {
        self.sprite_sheets
            .iter()
            .find(|(sprite_sheet_name, _, _)| sprite_sheet_name == name)
            .map(|(_, path, dimensions)| (path.clone(), dimensions.clone()))
            .or_else(|| {
                info!(
                    "SpriteSheet {:?} not found. Using default...",
                    name.as_str()
                );
                let path = self.image_directory.join(name.as_str()).with_extension("png");
                Some((path, SpriteSheetDimensions::new(1, 1)))
            })
            .unwrap()
    }
}

create_name_struct!(SpriteSheetName);
create_name_struct!(WindowName);
