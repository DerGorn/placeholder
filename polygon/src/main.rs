use ferride_core::{
    app::{ManagerApplication, WindowDescriptor},
    game_engine::{Game, RessourceDescriptorBuilder, Scene},
    graphics::{Index as I, RenderSceneDescriptor, ShaderDescriptor, Vertex},
    reexports::{
        wgpu::{vertex_attr_array, VertexAttribute},
        winit::PhysicalSize,
    },
};

mod sos;
mod polygon;
mod simple_polygon;
use simple_polygon::Polygon;
mod color;
use color::Color;

use ferride_core::game_engine::example::EmptyEntityType as EntityType;
use ferride_core::game_engine::example::EmptyExternalEvent as Event;
use ferride_core::game_engine::example::SimpleGameState;

use repr_trait::C;
use threed::Vector;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct SimpleVertex {
    position: [f32; 2],
    color: u32,
}
impl SimpleVertex {
    pub fn new(position: Vector<f32>, color: Color) -> Self {
        Self {
            position: [position.x, position.y],
            color: bytemuck::cast_slice(&color.to_slice())[0],
        }
    }
}
const UI_VERTEX_ATTRIBUTES: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x2, 1 => Uint32];
impl Vertex for SimpleVertex {
    fn attributes() -> &'static [VertexAttribute] {
        &UI_VERTEX_ATTRIBUTES
    }
}

const TARGET_FPS: u8 = 60;

fn main() {
    let ressources = RessourceDescriptorBuilder::new(RenderSceneDescriptor {
        index_format: u16::index_format(),
        use_textures: false,
        vertex_buffer_layout: SimpleVertex::describe_buffer_layout(),
    })
    .with_windows(vec![(
        "polygon".into(),
        WindowDescriptor::new()
            .with_title("Polygon")
            .with_inner_size(PhysicalSize::new(800, 600)),
    )])
    .build();

    let mut app = ManagerApplication::new(Game::new(
        ressources,
        TARGET_FPS,
        SimpleGameState::new(Scene {
            name: "polygon".into(),
            render_scene: "polygon".into(),
            target_window: "polygon".into(),
            shader_descriptor: ShaderDescriptor {
                file: "C:\\Users\\DarkL\\Desktop\\Rust\\placeholder\\polygon\\shader.wgsl",
                fragment_shader: "fs_main",
                vertex_shader: "vs_main",
                uniforms: &[],
            },
            z_index: 0,
            entities: vec![Box::new(Polygon::default())],
        }),
    ));
    app.run();
}
