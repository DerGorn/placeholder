use std::path::PathBuf;

use placeholder::{app::WindowDescriptor, graphics::ShaderDescriptor};

use super::sprite_sheet::SpriteSheetDimensions;

pub struct RessourceDescriptor {
    pub windows: Vec<(WindowName, WindowDescriptor, ShaderDescriptor)>,
    pub sprite_sheets: Vec<(SpriteSheetName, PathBuf, SpriteSheetDimensions)>,
}
impl RessourceDescriptor {
    pub fn get_window(&self, name: &WindowName) -> Option<(WindowDescriptor, ShaderDescriptor)> {
        self.windows
            .iter()
            .find(|(window_name, _, _)| window_name == name)
            .map(|(_, window, shader)| (window.clone(), shader.clone()))
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

macro_rules! create_name_struct {
    ($name: ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name(String);
        impl $name {
            #[allow(dead_code)]
            pub fn as_str<'a>(&'a self) -> &'a str {
                self.0.as_str()
            }
        }
        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }
        impl From<String> for $name {
            fn from(value: String) -> Self {
                value.as_str().into()
            }
        }
        impl From<&String> for $name {
            fn from(value: &String) -> Self {
                value.as_str().into()
            }
        }
    };
}
create_name_struct!(SpriteSheetName);
create_name_struct!(WindowName);
