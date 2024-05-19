use placeholder::app::{
    ApplicationEvent, EventManager, ManagerApplication, ShaderDescriptor, Vertex, WindowDescriptor,
    WindowManager,
};
use winit::window::{WindowAttributes, WindowId};

#[derive(Debug)]
enum Event {
    Resumed,
    NewWindow(WindowId),
    RequestNewWindow(WindowDescriptor, ShaderDescriptor),
    RenderUpdate(WindowId, Vec<Vertex>, Vec<u16>),
}
impl ApplicationEvent for Event {
    fn app_resumed() -> Self {
        Self::Resumed
    }

    fn is_request_new_window<'a>(&'a self) -> Option<(&'a WindowDescriptor, &'a ShaderDescriptor)> {
        if let Self::RequestNewWindow(window_descriptor, shader_descriptor) = self {
            Some((&window_descriptor, &shader_descriptor))
        } else {
            None
        }
    }

    fn is_render_update<'a>(
        &'a self,
    ) -> Option<(
        &'a winit::window::WindowId,
        Option<&'a [placeholder::app::Vertex]>,
        Option<&'a [u16]>,
    )> {
        if let Self::RenderUpdate(id, vertices, indices) = self {
            Some((
                &id,
                if vertices.len() > 0 {
                    Some(&vertices)
                } else {
                    None
                },
                if indices.len() > 0 {
                    Some(&indices)
                } else {
                    None
                },
            ))
        } else {
            None
        }
    }

    fn new_window(id: &WindowId) -> Self {
        Self::NewWindow(id.clone())
    }
}
struct EventHandler {
    default_window: WindowDescriptor,
}
impl EventManager<Event> for EventHandler {
    fn window_event(
        &mut self,
        window_manager: &mut WindowManager<Event>,
        event_loop: &winit::event_loop::ActiveEventLoop,
        id: &winit::window::WindowId,
        event: &winit::event::WindowEvent,
    ) -> bool
    where
        Self: Sized,
    {
        // todo!()
        true
    }

    fn user_event(
        &mut self,
        window_manager: &mut WindowManager<Event>,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: &Event,
    ) where
        Self: Sized,
    {
        match event {
            Event::Resumed => {
                let descriptor = self.default_window.clone();
                let shader_descriptor = ShaderDescriptor {
                    file: "res/shader/shader.wgsl",
                    vertex_shader: "vs_main",
                    fragment_shader: "fs_main",
                };
                window_manager
                    .send_event(Event::RequestNewWindow(
                        descriptor.clone(),
                        shader_descriptor.clone(),
                    ))
                    .unwrap();
                window_manager
                    .send_event(Event::RequestNewWindow(
                        descriptor,
                        shader_descriptor,
                    ))
                    .unwrap()
            }
            _ => {}
        }
    }
}

fn main() {
    let cursor_path = "res/images/cursor/Cursor_Goth_Cursor.png";
    let default_window =
        WindowAttributes::default().with_title("Wispers in the Void - Dark Dynasty");
    let default_window = WindowDescriptor::new(default_window).with_cursor(cursor_path);
    let mut app = ManagerApplication::new(EventHandler { default_window });
    app.run();
}
