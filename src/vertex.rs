use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, SpritePosition, SpriteSheet, TextureCoordinates},
    graphics::Vertex as Vert,
};
use repr_trait::C;
use threed::Vector;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    texture: u32,
}
impl Vertex {
    pub fn new(
        position: Vector<f32>,
        texture_coordinates: &TextureCoordinates,
        texture: u32,
    ) -> Self {
        Self {
            position: [position.x, position.y],
            tex_coords: [texture_coordinates.u, texture_coordinates.v],
            texture,
        }
    }
}
impl Vert for Vertex {
    fn describe_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Uint32,
                    shader_location: 2,
                },
            ],
        }
    }
}

pub fn render_sprite(
    bounding_box: &BoundingBox,
    vertices: &mut VertexBuffer,
    indices: &mut IndexBuffer,
    sprite_sheet: &SpriteSheet,
    sprite_position: &SpritePosition,
) {
    let y = bounding_box.anchor.y;
    let x = bounding_box.anchor.x;
    let x_offset = bounding_box.size.width / 2.0;
    let y_offset = bounding_box.size.height / 2.0;
    let texture_coords = sprite_sheet.get_sprite_coordinates(sprite_position);
    let new_vertices = [
        Vertex::new(
            Vector::new(x - x_offset, y + y_offset, 0.0),
            &texture_coords[0],
            sprite_sheet.texture(),
        ),
        Vertex::new(
            Vector::new(x + x_offset, y + y_offset, 0.0),
            &texture_coords[1],
            sprite_sheet.texture(),
        ),
        Vertex::new(
            Vector::new(x + x_offset, y - y_offset, 0.0),
            &texture_coords[2],
            sprite_sheet.texture(),
        ),
        Vertex::new(
            Vector::new(x - x_offset, y - y_offset, 0.0),
            &texture_coords[3],
            sprite_sheet.texture(),
        ),
    ];
    let start_index = vertices.len() as u16;
    let new_indices = [
        start_index,
        start_index + 1,
        start_index + 2,
        start_index,
        start_index + 2,
        start_index + 3,
    ];
    vertices.extend_from_slice(&new_vertices);
    indices.extend_from_slice(&new_indices);
}
