use placeholder::app::{EventManager, ManagerApplication, WindowDescriptor, WindowManager};
use winit::window::WindowAttributes;

struct EventHandler {}
impl EventManager<()> for EventHandler {
    fn window_event(
        &mut self,
        window_manager: &mut WindowManager<()>,
        event_loop: &winit::event_loop::ActiveEventLoop,
        id: winit::window::WindowId,
        event: winit::window::WindowId,
    ) -> bool
    where
        Self: Sized,
    {
        // todo!()
        true
    }
}



fn main() {
    let cursor_path = "res/images/cursor/Cursor_Goth_Cursor.png";
    let default_window = WindowAttributes::default().with_title("Wispers in the Void - Dark Dynasty");
    let default_window = WindowDescriptor::new(default_window).with_cursor(cursor_path);
    let mut app = ManagerApplication::new(EventHandler {}, Some(default_window));
    app.run();
}
