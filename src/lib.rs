mod graphics_provider;
pub mod graphics {
    pub use super::graphics_provider::{Vertex, Index, ShaderDescriptor};
}

mod manager_application;
pub mod app {
    pub use super::manager_application::{
        ApplicationEvent, EventManager, ManagerApplication, WindowDescriptor,
        WindowManager,
    };
}
