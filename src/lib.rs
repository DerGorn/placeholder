mod manager_application;
pub mod app {
    pub use super::manager_application::{
        EventManager, ManagerApplication, WindowDescriptor, WindowManager,
    };
}
