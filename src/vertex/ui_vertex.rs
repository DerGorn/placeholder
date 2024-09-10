use ferride_core::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, SpritePosition, SpriteSheet, TextureCoordinates},
    graphics::Vertex,
};
use repr_trait::C;
use threed::Vector;
use ferride_core::reexports::wgpu::vertex_attr_array;

use crate::color::Color;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct UiVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    texture: u32,
    color: u32,
}
impl UiVertex {
    pub fn new(
        position: &Vector<f32>,
        texture_coordinates: &TextureCoordinates,
        texture: u32,
        color: Color,
    ) -> Self {
        let color = color.to_slice();
        let color = u32::from_be_bytes(color);
        Self {
            position: [position.x, position.y],
            tex_coords: [texture_coordinates.u, texture_coordinates.v],
            texture,
            color,
        }
    }
}
const UI_VERTEX_ATTRIBUTES: [ferride_core::reexports::wgpu::VertexAttribute; 4] =
    vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32, 3 => Uint32];
impl Vertex for UiVertex {
    fn attributes() -> &'static [ferride_core::reexports::wgpu::VertexAttribute] {
        &UI_VERTEX_ATTRIBUTES
    }
}

pub fn render_ui_box_border(
    bounding_box: &BoundingBox,
    vertices: &mut VertexBuffer,
    indices: &mut IndexBuffer,
    border_thickness: f32,
    color: &Color,
) {
    let y = bounding_box.anchor.y;
    let x = bounding_box.anchor.x;
    let x_offset = bounding_box.size.width / 2.0;
    let y_offset = bounding_box.size.height / 2.0;
    let sprite_sheet = SpriteSheet::default();
    let texture_coords = sprite_sheet.get_sprite_coordinates(&SpritePosition::new(0, 0));
    let squares = [
        [
            Vector::new(x - x_offset - border_thickness, y + y_offset, 0.0),
            Vector::new(x - x_offset, y + y_offset, 0.0),
            Vector::new(x - x_offset, y - y_offset, 0.0),
            Vector::new(x - x_offset - border_thickness, y - y_offset, 0.0),
        ],
        [
            Vector::new(x - x_offset, y + y_offset + border_thickness, 0.0),
            Vector::new(x + x_offset, y + y_offset + border_thickness, 0.0),
            Vector::new(x + x_offset, y + y_offset, 0.0),
            Vector::new(x - x_offset, y + y_offset, 0.0),
        ],
        [
            Vector::new(x + x_offset, y + y_offset, 0.0),
            Vector::new(x + x_offset + border_thickness, y + y_offset, 0.0),
            Vector::new(x + x_offset + border_thickness, y - y_offset, 0.0),
            Vector::new(x + x_offset, y - y_offset, 0.0),
        ],
        [
            Vector::new(x - x_offset, y - y_offset, 0.0),
            Vector::new(x + x_offset, y - y_offset, 0.0),
            Vector::new(x + x_offset, y - y_offset - border_thickness, 0.0),
            Vector::new(x - x_offset, y - y_offset - border_thickness, 0.0),
        ],
    ];
    for square in &squares {
        let new_vertices = [
            UiVertex::new(
                &square[0],
                &texture_coords[0],
                sprite_sheet.texture(),
                color.clone(),
            ),
            UiVertex::new(
                &square[1],
                &texture_coords[1],
                sprite_sheet.texture(),
                color.clone(),
            ),
            UiVertex::new(
                &square[2],
                &texture_coords[2],
                sprite_sheet.texture(),
                color.clone(),
            ),
            UiVertex::new(
                &square[3],
                &texture_coords[3],
                sprite_sheet.texture(),
                color.clone(),
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
    let corner_triangles = [[0, 4, 1], [5, 9, 6], [11, 10, 14], [3, 2, 15]];
    for triangle in corner_triangles {
        let mut vectors = triangle.iter().map(|i| {
            let square = &squares[*i / 4];
            &square[*i % 4]
        });
        let new_vertices = [
            UiVertex::new(
                vectors.next().expect("Unreachable"),
                &texture_coords[0],
                sprite_sheet.texture(),
                color.clone(),
            ),
            UiVertex::new(
                vectors.next().expect("Unreachable"),
                &texture_coords[1],
                sprite_sheet.texture(),
                color.clone(),
            ),
            UiVertex::new(
                vectors.next().expect("Unreachable"),
                &texture_coords[2],
                sprite_sheet.texture(),
                color.clone(),
            ),
        ];
        let start_index = vertices.len() as u16;
        let new_indices = [start_index, start_index + 1, start_index + 2];
        vertices.extend_from_slice(&new_vertices);
        indices.extend_from_slice(&new_indices);
    }
}

const NO_BLEND_COLOR: Color = Color::new_rgba(0, 0, 0, 0);
pub fn render_ui_sprite(
    bounding_box: &BoundingBox,
    vertices: &mut VertexBuffer,
    indices: &mut IndexBuffer,
    sprite_sheet: &SpriteSheet,
    sprite_position: &SpritePosition,
    color: Option<&Color>,
) {
    let y = bounding_box.anchor.y;
    let x = bounding_box.anchor.x;
    let x_offset = bounding_box.size.width / 2.0;
    let y_offset = bounding_box.size.height / 2.0;
    let texture_coords = sprite_sheet.get_sprite_coordinates(sprite_position);
    let color = color.unwrap_or(&NO_BLEND_COLOR);
    let new_vertices = [
        UiVertex::new(
            &Vector::new(x - x_offset, y + y_offset, 0.0),
            &texture_coords[0],
            sprite_sheet.texture(),
            color.clone(),
        ),
        UiVertex::new(
            &Vector::new(x + x_offset, y + y_offset, 0.0),
            &texture_coords[1],
            sprite_sheet.texture(),
            color.clone(),
        ),
        UiVertex::new(
            &Vector::new(x + x_offset, y - y_offset, 0.0),
            &texture_coords[2],
            sprite_sheet.texture(),
            color.clone(),
        ),
        UiVertex::new(
            &Vector::new(x - x_offset, y - y_offset, 0.0),
            &texture_coords[3],
            sprite_sheet.texture(),
            color.clone(),
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
