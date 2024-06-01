mod graphics_provider;
pub mod graphics {
    pub use super::graphics_provider::{Vertex, Index, ShaderDescriptor, GraphicsProvider, UniformBufferName};
}

mod manager_application;
pub mod app {
    pub use super::manager_application::{
        ApplicationEvent, EventManager, ManagerApplication, WindowDescriptor,
        WindowManager,
    };
}

#[macro_export]
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
