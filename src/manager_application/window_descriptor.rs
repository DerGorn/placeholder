use image::imageops::{resize, FilterType};
use std::fs;
use winit::{
    event_loop::ActiveEventLoop,
    window::{CustomCursor, CustomCursorSource, Icon, WindowAttributes},
};

#[derive(Clone, Debug)]
pub struct WindowDescriptor {
    attributes: WindowAttributes,
    cursor_path: Option<&'static str>,
    icon_path: Option<&'static str>,
}
impl WindowDescriptor {
    pub fn new(attributes: WindowAttributes) -> Self {
        Self {
            attributes,
            ..Default::default()
        }
    }

    pub fn with_cursor(mut self, path: &'static str) -> Self {
        self.cursor_path = Some(path);
        self
    }

    pub fn with_icon(mut self, path: &'static str) -> Self {
        self.icon_path = Some(path);
        self
    }

    fn decode_icon(&self, path: &'static str) -> Icon {
        let bytes = fs::read(path).expect(&format!("Could not read icon file at '{}'", path));

        let (icon_rgba, icon_width, icon_height) = {
            let image = image::load_from_memory(&bytes)
                .expect(&format!("Could not parse icon file at '{}'", path))
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        Icon::from_rgba(icon_rgba, icon_width, icon_height)
            .expect(&format!("Could not make icon from file at '{}'", path))
    }

    fn decode_cursor(&self, path: &'static str) -> CustomCursorSource {
        let bytes = fs::read(path).expect(&format!("Could not read cursor file at '{}'", path));
        let img = image::load_from_memory(&bytes)
            .expect(&format!("Could not parse cursor file at '{}'", path))
            .into_rgba8();
        let img = resize(&img, 32, 32, FilterType::Gaussian);
        let samples = img.into_flat_samples();
        let (_, w, h) = samples.extents();
        let (w, h) = (w as u16, h as u16);
        CustomCursor::from_rgba(samples.samples, w, h, w / 4, 0)
            .expect(&format!("Could not make cursor from file at '{}'", path))
    }

    pub fn get_attributes(&self, event_loop: &ActiveEventLoop) -> WindowAttributes {
        let mut attributes = self.attributes.clone();
        if let Some(cursor_path) = self.cursor_path {
            let cursor_source = self.decode_cursor(cursor_path);
            attributes = attributes.with_cursor(event_loop.create_custom_cursor(cursor_source));
        }
        if let Some(icon_path) = self.icon_path {
            let icon = self.decode_icon(icon_path);
            attributes = attributes.with_window_icon(Some(icon));
        }
        attributes
    }
}
impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            attributes: WindowAttributes::default(),
            cursor_path: None,
            icon_path: None,
        }
    }
}
