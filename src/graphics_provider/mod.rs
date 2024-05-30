#![allow(deprecated)]
use std::fs;
use std::path::Path;

use wgpu::rwh::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::util::DeviceExt;
use winit::window::{Window, WindowId};

mod buffer_primitives;
pub use buffer_primitives::{Index, Vertex};

mod surface;
use surface::{Surface, WindowSurface};

mod shader_descriptor;
pub use shader_descriptor::ShaderDescriptor;

mod texture;
use texture::TextureProvider;

pub struct GraphicsProvider<I: Index, V: Vertex> {
    instance: wgpu::Instance,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surfaces: Vec<(WindowId, Box<dyn WindowSurface<I, V>>)>,
    texture_provider: Option<TextureProvider>,
}
impl<I: Index, V: Vertex> GraphicsProvider<I, V> {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        Self {
            instance,
            adapter: None,
            device: None,
            queue: None,
            surfaces: Vec::new(),
            texture_provider: None,
        }
    }

    fn init(&mut self, surface: &wgpu::Surface) {
        let adapter = futures::executor::block_on(self.instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ))
        .expect("Buy a new GPU. Not all prerequisites met");

        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::TEXTURE_BINDING_ARRAY
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        ))
        .expect("Buy a new GPU. Not all prerequisites met");
        self.texture_provider = Some(TextureProvider::new());
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
    }

    pub fn init_window(&mut self, window: &Window, shader_descriptor: &ShaderDescriptor) {
        let size = window.inner_size();
        //#Safety
        //
        //Should be safe if surface discarded when window is destroyed
        let surface = unsafe {
            self.instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: window
                        .raw_display_handle()
                        .expect("The window has no display handle"),
                    raw_window_handle: window
                        .raw_window_handle()
                        .expect("The window has no window handle"),
                })
        }
        .expect("Could not create a surface");

        if self.adapter.is_none() {
            self.init(&surface);
        }

        let capabilities = surface.get_capabilities(
            &self
                .adapter
                .as_ref()
                .expect("The surface is not compatible with the adapter"),
        );
        let format = capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .or(Some(capabilities.formats[0]))
            .expect("No compatible format found");
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let device = self.device.as_ref().expect("The device vanished");
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Vertex Buffer {:?}", window.id())),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Index Buffer {:?}", window.id())),
            contents: &[],
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        let num_vertices = 0;
        let num_indices = 0;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Shader Module {:?}", shader_descriptor.file)),
            source: wgpu::ShaderSource::Wgsl(
                fs::read_to_string(shader_descriptor.file)
                    .expect(&format!("Could not load '{}'\n", shader_descriptor.file))
                    .into(),
            ),
        });

        self.surfaces.push((
            window.id(),
            Box::new(Surface {
                wgpu_surface: surface,
                size,
                config,
                render_pipeline: None,
                shader,
                shader_descriptor: shader_descriptor.clone(),
                vertex_buffer,
                index_buffer,
                num_vertices,
                num_indices,
                _phantom: std::marker::PhantomData,
            }),
        ));
    }

    pub fn resize_window(&mut self, id: &WindowId, new_size: &winit::dpi::PhysicalSize<u32>) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let Some(device) = &self.device {
                surface.resize(new_size, device);
            }
        }
    }

    pub fn render_window(&mut self, id: &WindowId) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let (Some(device), Some(queue), Some(texture_provider)) =
                (&self.device, &self.queue, &self.texture_provider)
            {
                surface.render(device, queue, texture_provider);
            }
        }
    }

    pub fn update_buffers(&mut self, id: &WindowId, vertices: Option<&[V]>, indices: Option<&[I]>) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
                surface.update(device, queue, vertices, indices)
            }
        }
    }

    pub fn remove_window(&mut self, id: &WindowId) {
        self.surfaces.retain(|(i, _)| i != id);
    }

    pub fn create_texture(&mut self, path: &Path, label: &str) -> Option<u32> {
        if let (Some(device), Some(queue), Some(texture_provider)) =
            (&self.device, &self.queue, &mut self.texture_provider)
        {
            if let Some(index) = texture_provider.get_texture_index(label) {
                return Some(index);
            }
            let index = texture_provider.create_texture(device, queue, path, label);
            self.surfaces.iter_mut().for_each(|(_, surface)| {
                surface.create_render_pipeline(
                    device,
                    texture_provider
                        .bind_group_layout
                        .as_ref()
                        .expect("No bind group layout"),
                );
            });
            Some(index)
        } else {
            None
        }
    }
}
