use crate::{
    color::Color,
    sos::{FixedNumber, Point, SCALE},
    EntityType, Event, SimpleVertex,
};

use ferride_core::{
    game_engine::{BoundingBox, Entity, EntityName},
    reexports::winit::PhysicalSize,
};
use threed::{Matrix, Vector};

type Triangle = [usize; 3];
/// Index into vertices
type VertexIndex = usize;
struct Edge {
    start: VertexIndex,
    end: VertexIndex,
}
impl Edge {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
impl std::fmt::Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.start, self.end)
    }
}
/// Index into edges
type EdgeIndex = usize;
pub struct Polygon {
    name: EntityName,
    vertices: Vec<Point>,
    edges: Vec<Edge>,
    triangles: Vec<Triangle>,

    ears: Vec<VertexIndex>,
    reflex: Vec<VertexIndex>,
    remaining_vertices: Vec<VertexIndex>,
}
impl Default for Polygon {
    fn default() -> Self {
        let mut polygon = Self {
            name: "polygon".into(),
            vertices: vec![
            Point::from_f32(0.03178571,  0.96428558),
            Point::from_f32(0.03178571,  0.70357117),
            Point::from_f32(0.10714286,  0.70357117),
            Point::from_f32(0.16410715,  0.72160706),
            Point::from_f32(0.18500000,  0.77107117),
            Point::from_f32(0.17232143,  0.80732117),
            Point::from_f32(0.13964287,  0.82678589),
            Point::from_f32(0.13964287,  0.82785706),
            Point::from_f32(0.16625000,  0.83857117),
            Point::from_f32(0.18446430,  0.86125000),
            Point::from_f32(0.19107143,  0.89142853),
            Point::from_f32(0.16982143,  0.94464294),
            Point::from_f32(0.11214286,  0.96428558),
                            ///////////////////////////////// X
                            // Point::from_f32(0.037, 0.0),
                            // Point::from_f32(0.141, 0.1855),
                            // Point::from_f32(0.245, 0.371),
                            // Point::from_f32(0.1465, 0.5505),
                            // Point::from_f32(0.048, 0.730),
                            // Point::from_f32(0.1045, 0.730),
                            // Point::from_f32(0.161, 0.730),
                            // Point::from_f32(0.215, 0.626),
                            // Point::from_f32(0.269, 0.522),
                            // Point::from_f32(0.278, 0.504),
                            // Point::from_f32(0.2875, 0.485),
                            // Point::from_f32(0.297, 0.466),
                            // Point::from_f32(0.302, 0.456),
                            // Point::from_f32(0.307, 0.466),
                            // Point::from_f32(0.316, 0.485),
                            // Point::from_f32(0.325, 0.504),
                            // Point::from_f32(0.334, 0.522),
                            // Point::from_f32(0.3885, 0.626),
                            // Point::from_f32(0.443, 0.730),
                            // Point::from_f32(0.4975, 0.730),
                            // Point::from_f32(0.552, 0.730),
                            // Point::from_f32(0.4535, 0.553),
                            // Point::from_f32(0.355, 0.376),
                            // Point::from_f32(0.459, 0.188),
                            // Point::from_f32(0.563, 0.0),
                            // Point::from_f32(0.507, 0.0),
                            // Point::from_f32(0.451, 0.0),
                            // Point::from_f32(0.391, 0.112),
                            // Point::from_f32(0.331, 0.224),
                            // Point::from_f32(0.322, 0.242),
                            // Point::from_f32(0.313, 0.2615),
                            // Point::from_f32(0.304, 0.281),
                            // Point::from_f32(0.299, 0.293),
                            // Point::from_f32(0.294, 0.281),
                            // Point::from_f32(0.285, 0.2615),
                            // Point::from_f32(0.276, 0.242),
                            // Point::from_f32(0.266, 0.225),
                            // Point::from_f32(0.206, 0.1125),
                            // Point::from_f32(0.146, 0.0),
                        ],
            edges: vec![
                Edge::new(0, 1),
                Edge::new(1, 2),
                Edge::new(2, 3),
                Edge::new(3, 4),
                Edge::new(4, 5),
                Edge::new(5, 6),
                Edge::new(6, 7),
                Edge::new(7, 8),
                Edge::new(8, 9),
                Edge::new(9, 10),
                Edge::new(10, 11),
                Edge::new(11, 12),
                Edge::new(12, 0),
            ],
            triangles: vec![],

            ears: vec![],
            reflex: vec![],
            remaining_vertices: vec![],
        };
        polygon.remaining_vertices = (0..polygon.vertices.len()).collect::<Vec<_>>();
        polygon.triangulate();
        polygon
    }
}
fn triangle_sign(triangle: [&Point; 3]) -> i128 {
    let a = triangle[0];
    let b = triangle[1];
    let c = triangle[2];
    (a.x() - c.x()) * (b.y() - c.y()) - (b.x() - c.x()) * (a.y() - c.y())
}
fn is_point_in_triangle(point: &Point, triangle: &[&Point; 3]) -> bool {
    let sign1 = triangle_sign([point, triangle[0], triangle[1]]);
    let sign2 = triangle_sign([point, triangle[1], triangle[2]]);
    let sign3 = triangle_sign([point, triangle[2], triangle[0]]);

    !(((sign1 < 0) || (sign2 < 0) || (sign3 < 0)) && ((sign1 > 0) || (sign2 > 0) || (sign3 > 0)))
}
impl Polygon {
    /// Direction of first line is irrelevant as long as the reletive position of the second line
    /// is confirmed first. Because it is horizontal, it will either cancel out or not needed.
    fn horizontal_edge_intersection(&self, origin: VertexIndex, edge: &Edge) -> Point {
        let origin = &self.vertices[origin];
        let origin2 = &self.vertices[edge.start];
        let e2 = &self.vertices[edge.end];
        let direction2 = Point::from_i128(e2.x() - origin2.x(), e2.y() - origin2.y());

        if direction2.x.value == 0 {
            Point::from_i128(origin2.x(), origin.y())
        } else {
            let r = (origin.y.value - origin2.y.value) / direction2.y.value;
            Point::new(
                origin2.x.value + direction2.x.value * r,
                origin2.y.value + direction2.y.value * r,
            )
        }
    }
    fn find_possible_horizontal_intersecting_edges<'a>(
        &'a self,
        origin: &Point,
        origin_index: VertexIndex,
        possible_vertices: impl Iterator<Item = (VertexIndex, &'a Point)>,
    ) -> Vec<EdgeIndex> {
        possible_vertices
            .map(|(j, _)| j)
            .fold(vec![], |mut edges, j| {
                for (edge_index, edge) in self.edges.iter().enumerate() {
                    if (edge.start == j && edge.end != origin_index)
                        || (edge.end == j && edge.start != origin_index)
                    {
                        let start = &self.vertices[edge.start];
                        let end = &self.vertices[edge.end];
                        if (start.y.smaller(edge.start, &origin.y, origin_index)
                            && end.y.smaller(edge.end, &origin.y, origin_index))
                            || (!start.y.smaller(edge.start, &origin.y, origin_index)
                                && !end.y.smaller(edge.end, &origin.y, origin_index))
                        {
                            continue;
                        }
                        if !edges.contains(&edge_index) {
                            edges.push(edge_index);
                        }
                    }
                }
                edges
            })
    }
    fn open_holes(&mut self) {
        for (i, vertex) in self.vertices.iter().enumerate() {
            // println!("\n------------------------------");
            // println!("Vertex {i}: {:?}", vertex);
            let right_vertices = self
                .vertices
                .iter()
                .enumerate()
                .filter(|(j, v)| i != *j && !v.x.smaller(*j, &vertex.x, i));
            // println!(
            //     "Right vertices: {:#?}",
            //     right_vertices.clone().collect::<Vec<_>>()
            // );
            let right_edges =
                self.find_possible_horizontal_intersecting_edges(vertex, i, right_vertices.clone());
            // println!("Right edges: {:?}", right_edges);

            let left_vertices = self
                .vertices
                .iter()
                .enumerate()
                .filter(|(j, v)| i != *j && v.x.smaller(*j, &vertex.x, i));
            // println!(
            //     "Left vertices: {:#?}",
            //     left_vertices.clone().collect::<Vec<_>>()
            // );
            let left_edges =
                self.find_possible_horizontal_intersecting_edges(vertex, i, left_vertices.clone());
            // println!("Left edges: {:?}", left_edges);

            // println!("Right edges: {:?}", right_edges);
            let right_intersection = right_edges
                .iter()
                .filter_map(|e| {
                    // println!("edge {e}: {:?}", self.edges[*e]);
                    let intersection = self.horizontal_edge_intersection(i, &self.edges[*e]);
                    // println!("Intersection: {:?}", intersection);
                    if intersection.x.value > vertex.x.value {
                        Some((e, intersection))
                    } else {
                        None
                    }
                })
                .fold(
                    (0_usize, Point::from_i128(i128::MAX, 0)),
                    |(e_min, min_i), (e, i)| {
                        if i.x.value < min_i.x.value {
                            (*e, i)
                        } else {
                            (e_min, min_i)
                        }
                    },
                );
            // println!("Right Intersection: {:?}", right_intersection);
            todo!("Right Intersection");

            // println!("Left edges: {:?}", right_edges);
            let left_intersection = left_edges
                .iter()
                .filter_map(|e| {
                    // println!("edge {e}: {:?}", self.edges[*e]);
                    let intersection = self.horizontal_edge_intersection(i, &self.edges[*e]);
                    // println!("Intersection: {:?}", intersection);
                    if intersection.x.value < vertex.x.value {
                        Some((e, intersection))
                    } else {
                        None
                    }
                })
                .fold(
                    (0_usize, Point::from_i128(i128::MIN, 0)),
                    |(e_max, max_i), (e, i)| {
                        if i.x.value > max_i.x.value {
                            (*e, i)
                        } else {
                            (e_max, max_i)
                        }
                    },
                );
            // println!("Left Intersection: {:?}", left_intersection);
            todo!("Left Intersection");
        }
    }
    fn get_neigbhour_vertices(&self, vertex_index: VertexIndex) -> (VertexIndex, VertexIndex) {
        let in_edge = self
            .edges
            .iter()
            .find(|edge| edge.end == vertex_index)
            .unwrap();
        let mut in_vertex_index = in_edge.start;
        let out_edge = self
            .edges
            .iter()
            .find(|edge| edge.start == vertex_index)
            .unwrap();
        let mut out_vertex_index = out_edge.end;

        while !self.remaining_vertices.contains(&in_vertex_index) {
            in_vertex_index = self
                .edges
                .iter()
                .find(|edge| edge.end == in_vertex_index)
                .unwrap()
                .start;
        }
        while !self.remaining_vertices.contains(&out_vertex_index) {
            out_vertex_index = self
                .edges
                .iter()
                .find(|edge| edge.start == out_vertex_index)
                .unwrap()
                .end;
        }

        (in_vertex_index, out_vertex_index)
    }
    fn is_vertex_reflex(&mut self, vertex_index: VertexIndex) -> bool {
        let (in_vertex_index, out_vertex_index) = self.get_neigbhour_vertices(vertex_index);
        let vertex = &self.vertices[vertex_index];
        let in_vertex = &self.vertices[in_vertex_index];
        let in_vector = Vector::new(
            vertex.x.value - in_vertex.x.value,
            vertex.y.value - in_vertex.y.value,
            0.into(),
        );
        let out_vertex = &self.vertices[out_vertex_index];
        let out_vector = Vector::new(
            out_vertex.x.value - vertex.x.value,
            out_vertex.y.value - vertex.y.value,
            0.into(),
        );

        let angle = out_vector.angle(&Vector::y_axis());
        let angle = if out_vector.x < 0 {
            std::f32::consts::PI * 2.0 - angle
        } else {
            angle
        };
        let test_vector = in_vector.rotate_around(angle, &Vector::z_axis());
        test_vector.x < 0
    }
    fn find_reflex_vertices(&mut self) {
        self.reflex.clear();
        for vertex_index in 0..self.vertices.len() {
            if self.is_vertex_reflex(vertex_index) {
                self.reflex.push(vertex_index);
            }
        }
    }
    /// Assuming vertex_index is not reflex
    fn is_vertex_ear(&mut self, vertex_index: VertexIndex) -> bool {
        let (in_vertex_index, out_vertex_index) = self.get_neigbhour_vertices(vertex_index);
        let vertex = &self.vertices[vertex_index];
        let in_vertex = &self.vertices[in_vertex_index];
        let out_vertex = &self.vertices[out_vertex_index];

        let triangle = [in_vertex, vertex, out_vertex];
        let mut is_ear = true;
        for reflex_index in &self.reflex {
            let reflex_vertex = &self.vertices[*reflex_index];
            if reflex_index != &vertex_index
                && reflex_index != &in_vertex_index
                && reflex_index != &out_vertex_index
                && is_point_in_triangle(reflex_vertex, &triangle)
            {
                is_ear = false;
                break;
            }
        }
        is_ear
    }
    fn find_ears(&mut self) {
        self.ears.clear();
        for vertex_index in 0..self.vertices.len() {
            if self.reflex.contains(&vertex_index) {
                continue;
            }

            if self.is_vertex_ear(vertex_index) {
                self.ears.push(vertex_index);
            }
        }
    }
    fn prepare_polygon(&mut self) {
        // self.open_holes();
        self.find_reflex_vertices();
        self.find_ears();
    }
    fn triangulate(&mut self) {
        self.prepare_polygon();
        self.triangles.clear();
        self.remaining_vertices = (0..self.vertices.len()).collect::<Vec<_>>();
        while self.remaining_vertices.len() > 3 {
            let ear_index = match self.ears.pop() {
                Some(e) => e,
                None => break,
            };
            let (in_vertex_index, out_vertex_index) = self.get_neigbhour_vertices(ear_index);
            self.triangles
                .push([in_vertex_index, ear_index, out_vertex_index]);
            self.remaining_vertices.retain(|&v| v != ear_index);
            for vertex in [in_vertex_index, out_vertex_index].iter() {
                if self.reflex.contains(vertex) {
                    if self.is_vertex_reflex(*vertex) {
                        continue;
                    } else {
                        self.reflex.retain(|&v| v != *vertex);
                    }
                }
                if self.is_vertex_ear(*vertex) {
                    self.ears.push(*vertex);
                } else {
                    self.ears.retain(|&v| v != *vertex);
                }
            }
        }
        self.triangles.push([
            self.remaining_vertices[0],
            self.remaining_vertices[1],
            self.remaining_vertices[2],
        ]);
    }

    fn triangles(&self) -> impl IntoIterator<Item = ([Vector<f32>; 3], Color)> + '_ {
        self.edges
            .iter()
            .map(|edge| {
                let width = 0.004;
                let t = [edge.start, edge.start, edge.end];
                let orthogonal_x =
                    f32::from(self.vertices[edge.end].y.value - self.vertices[edge.start].y.value);
                let orthogonal_y =
                    f32::from(self.vertices[edge.end].x.value - self.vertices[edge.start].x.value);
                let max_ortho = orthogonal_x.abs().max(orthogonal_y.abs());
                let orthogonal_x = width * orthogonal_x / max_ortho;
                let orthogonal_y = -width * orthogonal_y / max_ortho;

                [
                    (
                        [
                            Vector::new(
                                f32::from(self.vertices[t[0]].x.value) + orthogonal_x,
                                f32::from(self.vertices[t[0]].y.value) + orthogonal_y,
                                0.0,
                            ),
                            Vector::new(
                                f32::from(self.vertices[t[1]].x.value) - orthogonal_x,
                                f32::from(self.vertices[t[1]].y.value) - orthogonal_y,
                                0.0,
                            ),
                            Vector::new(
                                f32::from(self.vertices[t[2]].x.value) - orthogonal_x,
                                f32::from(self.vertices[t[2]].y.value) - orthogonal_y,
                                0.0,
                            ),
                        ],
                        Color::from_str("red"),
                    ),
                    (
                        [
                            Vector::new(
                                f32::from(self.vertices[t[2]].x.value) - orthogonal_x,
                                f32::from(self.vertices[t[2]].y.value) - orthogonal_y,
                                0.0,
                            ),
                            Vector::new(
                                f32::from(self.vertices[t[2]].x.value) + orthogonal_x,
                                f32::from(self.vertices[t[2]].y.value) + orthogonal_y,
                                0.0,
                            ),
                            Vector::new(
                                f32::from(self.vertices[t[0]].x.value) + orthogonal_x,
                                f32::from(self.vertices[t[0]].y.value) + orthogonal_y,
                                0.0,
                            ),
                        ],
                        Color::from_str("red"),
                    ),
                ]
            })
            .flatten()
            .chain(self.vertices.iter().map(|v| {
                let width = 0.015;
                (
                    [
                        Vector::new(f32::from(v.x.value), f32::from(v.y.value) + width, 0.0),
                        Vector::new(
                            f32::from(v.x.value) - width,
                            f32::from(v.y.value) - width,
                            0.0,
                        ),
                        Vector::new(
                            f32::from(v.x.value) + width,
                            f32::from(v.y.value) - width,
                            0.0,
                        ),
                    ],
                    Color::from_str("black"),
                )
            }))
            .chain(self.triangles.iter().map(|t| {
                (
                    [
                        Vector::new(
                            f32::from(self.vertices[t[0]].x.value),
                            f32::from(self.vertices[t[0]].y.value),
                            0.0,
                        ),
                        Vector::new(
                            f32::from(self.vertices[t[1]].x.value),
                            f32::from(self.vertices[t[1]].y.value),
                            0.0,
                        ),
                        Vector::new(
                            self.vertices[t[2]].x.value.into(),
                            self.vertices[t[2]].y.value.into(),
                            0.0,
                        ),
                    ],
                    Color::from_str("purple"),
                )
            }))
    }
}
impl std::fmt::Debug for Polygon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Polygon").finish()
    }
}
impl Entity<EntityType, Event> for Polygon {
    fn render(
        &mut self,
        vertices: &mut ferride_core::app::VertexBuffer,
        indices: &mut ferride_core::app::IndexBuffer,
        _sprite_sheet: Vec<Option<&ferride_core::game_engine::SpriteSheet>>,
    ) {
        for (triangle, color) in self.triangles() {
            let new_vertices = [
                SimpleVertex::new(triangle[0].clone(), color.clone()),
                SimpleVertex::new(triangle[1].clone(), color.clone()),
                SimpleVertex::new(triangle[2].clone(), color.clone()),
            ];
            let start_index = vertices.len() as u16;
            let new_indices = [start_index, start_index + 1, start_index + 2];
            vertices.extend_from_slice(&new_vertices);
            indices.extend_from_slice(&new_indices)
        }
    }
    fn sprite_sheets(&self) -> Vec<&ferride_core::game_engine::SpriteSheetName> {
        vec![]
    }
    fn name(&self) -> &ferride_core::game_engine::EntityName {
        &self.name
    }
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            anchor: Vector::scalar(0.0),
            size: PhysicalSize::new(0.0, 0.0),
        }
    }
    fn entity_type(&self) -> EntityType {
        EntityType::Entity
    }
}
