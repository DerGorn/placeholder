use std::path::PathBuf;

use crate::app::WindowDescriptor;
use crate::create_name_struct;

use crate::game_engine::CameraDescriptor;

use super::sprite_sheet::SpriteSheetDimensions;

pub struct RessourceDescriptor {
    pub windows: Vec<(WindowName, WindowDescriptor, CameraDescriptor)>,
    pub sprite_sheets: Vec<(SpriteSheetName, PathBuf, SpriteSheetDimensions)>,
}
impl RessourceDescriptor {
    pub fn get_window(&self, name: &WindowName) -> Option<WindowDescriptor> {
        self.windows
            .iter()
            .find(|(window_name, _, _)| window_name == name)
            .map(|(_, window, _)| window.clone())
    }
    pub fn get_camera(&self, name: &WindowName) -> Option<CameraDescriptor> {
        self.windows
            .iter()
            .find(|(window_name, _, _)| window_name == name)
            .map(|(_, _, camera)| camera.clone())
    }
    pub fn get_sprite_sheet(
        &self,
        name: &SpriteSheetName,
    ) -> Option<(PathBuf, SpriteSheetDimensions)> {
        self.sprite_sheets
            .iter()
            .find(|(sprite_sheet_name, _, _)| sprite_sheet_name == name)
            .map(|(_, path, dimensions)| (path.clone(), dimensions.clone()))
    }
}

create_name_struct!(SpriteSheetName);
create_name_struct!(WindowName);
