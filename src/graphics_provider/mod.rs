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

use crate::create_name_struct;

use self::surface::RenderScene;
pub use self::surface::RenderSceneName;

create_name_struct!(UniformBufferName);

pub struct GraphicsProvider<I: Index, V: Vertex> {
    instance: wgpu::Instance,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    ///One to one relationship
    surfaces: Vec<(WindowId, Box<dyn WindowSurface>)>,
    ///One to many relationship
    render_scenes: Vec<(WindowId, RenderScene, wgpu::ShaderModule, ShaderDescriptor)>,
    uniform_buffers: Vec<(
        UniformBufferName,
        wgpu::Buffer,
        wgpu::BindGroupLayout,
        wgpu::BindGroup,
    )>,
    texture_provider: Option<TextureProvider>,
    _phantom: std::marker::PhantomData<(I, V)>,
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
            render_scenes: Vec::new(),
            uniform_buffers: Vec::new(),
            texture_provider: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn get_window(&self, render_scene: &RenderSceneName) -> Option<&WindowId> {
        self.render_scenes
            .iter()
            .find(|(_, scene, _, _)| render_scene == scene.name())
            .map(|(window_id, _, _, _)| window_id)
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
        self.texture_provider = Some(TextureProvider::new(&device, &queue));
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
    }

    pub fn init_window(&mut self, window: &Window) {
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

        self.surfaces.push((
            window.id(),
            Box::new(Surface {
                wgpu_surface: surface,
                size,
                config,
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
                let mut bind_groups =
                    vec![texture_provider.bind_group.as_ref().expect("No bind group")];
                bind_groups.extend(self.uniform_buffers.iter().map(|(_, _, _, bg)| bg));
                let render_scenes = self
                    .render_scenes
                    .iter()
                    .filter_map(|(i, s, _, _)| if i == id { Some(s) } else { None })
                    .collect::<Vec<_>>();
                surface.render(device, queue, &render_scenes, &bind_groups)
            }
        }
    }

    /// Update the vertex and index buffers of a window
    pub fn update_buffers(
        &mut self,
        render_scene: &RenderSceneName,
        vertices: Option<&[V]>,
        indices: Option<&[I]>,
    ) {
        if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
            for render_scene in self.render_scenes.iter_mut().filter_map(|(_, s, _, _)| {
                if render_scene == s.name() {
                    Some(s)
                } else {
                    None
                }
            }) {
                render_scene.update(device, queue, &vertices, &indices)
            }
        }
    }

    pub fn add_render_scene(
        &mut self,
        window_id: &WindowId,
        render_scene: RenderSceneName,
        shader_descriptor: ShaderDescriptor,
    ) {
        let device = self.device.as_ref().expect("The device vanished");
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Vertex Buffer {:?}", render_scene)),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Index Buffer {:?}", render_scene)),
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

        if let (Some((_, surface)), Some(texture_provider)) = (
            self.surfaces.iter().find(|(id, _)| id == window_id),
            &self.texture_provider,
        ) {
            let mut bind_groups_layouts = vec![texture_provider
                .bind_group_layout
                .as_ref()
                .expect("Default Texture vanished")];
            bind_groups_layouts.extend(
                self.uniform_buffers
                    .iter()
                    .map(|(_, _, bind_group_layout, _)| bind_group_layout),
            );
            let render_pipeline = surface.create_render_pipeline(
                device,
                &bind_groups_layouts,
                &shader,
                &shader_descriptor,
                V::describe_buffer_layout(),
            );
            let render_scene = RenderScene::new(
                render_scene,
                render_pipeline,
                vertex_buffer,
                index_buffer,
                num_vertices,
                num_indices,
                I::index_format(),
            );
            self.render_scenes
                .push((window_id.clone(), render_scene, shader, shader_descriptor));
        } else {
            panic!("No surface on window {:?}", window_id)
        }
    }

    pub fn remove_window(&mut self, id: &WindowId) {
        self.surfaces.retain(|(i, _)| i != id);
        self.render_scenes.retain(|(i, _, _, _)| i != id);
    }

    pub fn create_texture(
        &mut self,
        path: &Path,
        label: &str,
        render_scenes: &[RenderSceneName],
    ) -> Option<u32> {
        if let (Some(device), Some(queue), Some(texture_provider)) =
            (&self.device, &self.queue, &mut self.texture_provider)
        {
            let index = texture_provider.create_texture(device, queue, path, Some(label));
            let mut bind_groups_layouts = vec![texture_provider
                .bind_group_layout
                .as_ref()
                .expect("No texture bind group layout")];
            bind_groups_layouts.extend(
                self.uniform_buffers
                    .iter()
                    .map(|(_, _, bind_group_layout, _)| bind_group_layout),
            );
            self.render_scenes
                .iter_mut()
                .filter(|(_, s, _, _)| render_scenes.contains(&s.name()))
                .for_each(|(window_id, render_scene, shader, shader_descriptor)| {
                    if let Some((_, surface)) = self.surfaces.iter().find(|(id, _)| id == window_id)
                    {
                        let render_pipeline = surface.create_render_pipeline(
                            device,
                            &bind_groups_layouts,
                            shader,
                            shader_descriptor,
                            V::describe_buffer_layout(),
                        );
                        render_scene.update_pipeline(render_pipeline);
                    }
                });
            Some(index)
        } else {
            None
        }
    }

    pub fn create_uniform_buffer(
        &mut self,
        label: impl Into<UniformBufferName>,
        contents: &[u8],
        visibility: wgpu::ShaderStages,
    ) {
        let label: UniformBufferName = label.into();
        let device = self.device.as_ref().expect("The device vanished");
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label.as_str()),
            contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(label.as_str()),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label.as_str()),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        self.uniform_buffers
            .push((label.clone(), buffer, bind_group_layout, bind_group));
    }

    pub fn update_uniform_buffer(&self, label: &UniformBufferName, contents: &[u8]) {
        if let Some((_, buffer, _, _)) = self.uniform_buffers.iter().find(|(l, _, _, _)| l == label)
        {
            let queue = self.queue.as_ref().expect("The queue vanished");
            queue.write_buffer(buffer, 0, contents);
        }
    }
}
