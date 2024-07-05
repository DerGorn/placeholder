use placeholder::{
    app::{IndexBuffer, VertexBuffer},
    game_engine::{BoundingBox, SpriteSheet, TextureCoordinates},
};
use threed::Vector;

use crate::{color::Color, vertex::UiVertex};

const CHARACTER_PADDING: [u8; 95] = [
    6,  // Space
    11, // !
    7,  // "
    3,  // #
    2,  // $
    4,  // %
    4,  // &
    11, // '
    8,  // (
    8,  // )
    8,  // *
    8,  // +
    14, // ,
    8,  // -
    14, // .
    7,  // /
    7,  // 0
    8,  // 1
    6,  // 2
    6,  // 3
    6,  // 4
    6,  // 5
    6,  // 6
    6,  // 7
    6,  // 8
    6,  // 9
    10, // :
    10, // ;
    7,  // <
    8,  // =
    7,  // >
    5,  // ?
    2,  // @
    2,  // A
    6,  // B
    1,  // C
    4,  // D
    6,  // E
    6,  // F
    0,  // G
    7,  // H
    9,  // I
    7,  // J
    6,  // K
    5,  // L
    2,  // M
    5,  // N
    0,  // O
    6,  // P
    0,  // Q
    6,  // R
    3,  // S
    5,  // T
    4,  // U
    2,  // V
    2,  // W
    5,  // X
    5,  // Y
    4,  // Z
    9,  // [
    6,  // \
    9,  // ]
    7,  // ^
    2,  // _
    10, // `
    8,  // a
    8,  // b
    8,  // c
    8,  // d
    8,  // e
    9,  // f
    8,  // g
    9,  // h
    12, // i
    10, // j
    8,  // k
    9,  // l
    6,  // m
    8,  // n
    8,  // o
    8,  // p
    7,  // q
    10, // r
    8,  // s
    8,  // t
    8,  // u
    8,  // v
    6,  // w
    8,  // x
    8,  // y
    8,  // z
    3,  // {
    11, // |
    3,  // }
    9,  // ~
];
const PIXELS_PER_CHARACTER: f32 = 32.0;

pub fn render_character(
    c: char,
    color: &Color,
    bounding_box: &BoundingBox,
    vertices: &mut VertexBuffer,
    indices: &mut IndexBuffer,
    font: &SpriteSheet,
    // debug: bool,
) -> f32 {
    if (c as u32) < 32 || (c as u32) > 126 {
        panic!("Character not supported: {}", c);
    }
    let sub_ascii = c as u32 - 32;
    let x = (sub_ascii % 16) as u8;
    let y = (sub_ascii / 16) as u8;
    let padding_factor = CHARACTER_PADDING[sub_ascii as usize] as f32 / PIXELS_PER_CHARACTER;

    let width = 1.0 / font.sprites_per_row as f32;
    let height = 1.0 / font.sprites_per_column as f32;
    let padding_width = width * padding_factor;
    let x_offset = x as f32 * width + padding_width;
    let y_offset = y as f32 * height;
    let width = width - padding_width * 2.0;
    let texture_coords = [
        TextureCoordinates {
            u: x_offset,
            v: y_offset,
        },
        TextureCoordinates {
            u: x_offset + width,
            v: y_offset,
        },
        TextureCoordinates {
            u: x_offset + width,
            v: y_offset + height,
        },
        TextureCoordinates {
            u: x_offset,
            v: y_offset + height,
        },
    ];
    // if debug {
    //     println!(
    //         "texture_coords: {:?}",
    //         texture_coords
    //             .iter()
    //             .map(|tc| tc.u * (font.sprites_per_row as f32) * PIXELS_PER_CHARACTER)
    //             .collect::<Vec<f32>>()
    //     );
    // };

    let texture = font.texture();
    let y = bounding_box.anchor.y;
    let x = bounding_box.anchor.x;
    let x_offset = bounding_box.size.width * (1.0 - 2.0 * padding_factor) / 2.0;
    let y_offset = bounding_box.size.height / 2.0;
    // if debug {
    //     println!("padding: {}", CHARACTER_PADDING[sub_ascii as usize]);
    //     println!("padding_factor: {:?}", padding_factor);
    //     println!("bounding_box: {:?}", bounding_box);
    //     println!("x_offset: {:?}", x_offset);
    // };
    let new_vertices = [
        UiVertex::new(
            Vector::new(x - x_offset, y + y_offset, 0.0),
            &texture_coords[0],
            texture,
            color.clone(),
        ),
        UiVertex::new(
            Vector::new(x + x_offset, y + y_offset, 0.0),
            &texture_coords[1],
            texture,
            color.clone(),
        ),
        UiVertex::new(
            Vector::new(x + x_offset, y - y_offset, 0.0),
            &texture_coords[2],
            texture,
            color.clone(),
        ),
        UiVertex::new(
            Vector::new(x - x_offset, y - y_offset, 0.0),
            &texture_coords[3],
            texture,
            color.clone(),
        ),
    ];
    // if debug {
    //     println!("new_vertices: {:?}", new_vertices);
    // };
    let start_index = vertices.len() as u16;
    let new_indices = [
        start_index,
        start_index + 1,
        start_index + 2,
        start_index,
        start_index + 2,
        start_index + 3,
    ];
    println!("new_indices: {:?}", new_indices);
    println!("new_vertices: {:?}", new_vertices);
    println!("c: {}", c);
    vertices.extend_from_slice(&new_vertices);
    indices.extend_from_slice(&new_indices);
    x_offset * 2.0
}
