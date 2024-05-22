use winit::{event_loop::{EventLoopClosed, EventLoopProxy}, window::{Window, WindowId}};

pub struct WindowManager<E: 'static> {
    windows: Vec<Window>,
    event_loop: Option<EventLoopProxy<E>>,
}
impl<E: 'static> WindowManager<E> {
    pub fn set_event_loop(&mut self, event_loop: EventLoopProxy<E>) {
        self.event_loop = Some(event_loop);
    }

    pub fn send_event(&self, event: E) -> Result<(), EventLoopClosed<E>> {
        self.event_loop.as_ref().unwrap().send_event(event)
    }

    pub fn create_event_loop_proxy(&self) -> EventLoopProxy<E> {
        self.event_loop.as_ref().unwrap().clone()
    }

    pub fn amount_windows(&self) -> usize {
        self.windows.len()
    }

    pub fn get_window(&self, id: &WindowId) -> Option<&Window> {
        self.windows.iter().find(|window| window.id() == *id)
    }

    pub fn remove_window(&mut self, id: &WindowId) {
        self.windows.retain(|window| window.id() != *id)
    }

    pub fn add_window(&mut self, window: Window) {
        self.windows.push(window);
    }   
}
impl<E: 'static> Default for WindowManager<E> {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            event_loop: None,
        }
    }
}
