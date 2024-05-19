use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use super::WindowManager;

pub trait EventManager<E: 'static> {
    /// Handles window events in a WindowManager. Return `false` to prevent default behavior of the
    /// WindowManager. Default behavior is closing, resizing and rendering the window and toggling fullscreen on F11
    fn window_event(
        &mut self,
        window_manager: &mut WindowManager<E>,
        event_loop: &ActiveEventLoop,
        id: &WindowId,
        event: &WindowEvent,
    ) -> bool
    where
        Self: Sized;
    fn user_event(
        &mut self,
        _window_manager: &mut WindowManager<E>,
        _event_loop: &ActiveEventLoop,
        _event: &E,
    ) where
        Self: Sized,
    {
    }
}
