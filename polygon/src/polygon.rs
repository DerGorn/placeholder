use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use ferride_core::{
    game_engine::{BoundingBox, Entity, EntityName},
    reexports::winit::PhysicalSize,
};
use threed::Vector;

use crate::{color::Color, EntityType, Event, SimpleVertex};
#[derive(Clone, Copy, PartialEq, Default, PartialOrd)]
struct FixedNumber(i128);
impl Debug for FixedNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "F({})", f32::from(*self))
    }
}
const SCALE: i128 = 2 << 61;
impl Add for FixedNumber {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}
impl Sub for FixedNumber {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}
impl Mul for FixedNumber {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let result = self.0 * other.0 / SCALE;
        Self(result)
    }
}
impl Div for FixedNumber {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let result = self.0 * SCALE / rhs.0;
        Self(result)
    }
}
impl From<FixedNumber> for f32 {
    fn from(fixed: FixedNumber) -> Self {
        fixed.0 as f32 / SCALE as f32
    }
}
impl From<f32> for FixedNumber {
    fn from(f: f32) -> Self {
        Self((f * SCALE as f32) as i128)
    }
}

pub struct Coordinate {
    value: FixedNumber,
    index: usize,
}
impl Coordinate {
    fn smaller(&self, self_point: usize, other: &Self, other_point: usize) -> bool {
        if self.value != other.value {
            self.value < other.value
        } else if self_point != other_point {
            self_point > other_point
        } else {
            self.index < other.index
        }
    }
}
pub struct Point {
    x: Coordinate,
    y: Coordinate,
}
impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x.value.0, self.y.value.0)
    }
}
impl Point {
    fn new(x: FixedNumber, y: FixedNumber) -> Self {
        Self {
            x: Coordinate { value: x, index: 0 },
            y: Coordinate { value: y, index: 1 },
        }
    }
    fn from_f32(x: f32, y: f32) -> Self {
        Self {
            x: Coordinate {
                value: x.into(),
                index: 0,
            },
            y: Coordinate {
                value: y.into(),
                index: 1,
            },
        }
    }
}

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
#[derive(Debug)]
struct Cord {
    origin_vertex: VertexIndex,
    /// The edge the cord intersects with. If the origin is a global extemum, the cord intersects
    /// with orign after wrapping around and intersection is None
    intersection: Option<(EdgeIndex, Point)>,
    wrap_around: bool,
}
type CordIndex = usize;
#[derive(Debug)]
enum DoubleBoundaryPoint {
    Vertex(VertexIndex),
    CordEndpoint(CordIndex),
}
enum TraverseDirection {
    Clockwise,
    CounterClockwise,
}
struct ArcSegment {
    edge: EdgeIndex,
    traversal: TraverseDirection,
}
enum Arc {
    Short(ArcSegment),
    Arc { start: ArcSegment, end: ArcSegment },
}
pub struct Polygon {
    name: EntityName,
    vertices: Vec<Point>,
    edges: Vec<Edge>,
    triangles: Vec<Triangle>,

    cords: Vec<Cord>,
}
impl Default for Polygon {
    fn default() -> Self {
        let mut polygon = Self {
            name: "polygon".into(),
            vertices: vec![
                Point::from_f32(0.4, 0.3),
                Point::from_f32(0.2, 0.1),
                Point::from_f32(-0.2, 0.5),
                Point::from_f32(-0.5, -0.5),
                Point::from_f32(0.2, -0.2),
                Point::from_f32(-0.2, 0.2),
                Point::from_f32(-0.2, 0.0),
            ],
            edges: vec![
                Edge::new(0, 1),
                Edge::new(1, 2),
                Edge::new(2, 3),
                Edge::new(3, 4),
                Edge::new(4, 5),
                Edge::new(5, 6),
            ],
            triangles: vec![],
            cords: vec![],
        };
        polygon.triangulate();
        polygon
    }
}
impl Polygon {
    /// Direction of first line is irrelevant as long as the reletive position of the second line
    /// is confirmed first. Because it is horizontal, it will either cancel out or not needed.
    fn cord_edge_intersection(&self, origin: VertexIndex, edge: &Edge) -> Point {
        let origin = &self.vertices[origin];
        let origin2 = &self.vertices[edge.start];
        let e2 = &self.vertices[edge.end];
        let direction2 = Point::new(e2.x.value - origin2.x.value, e2.y.value - origin2.y.value);

        if direction2.x.value.0 == 0 {
            Point::new(origin2.x.value, origin.y.value)
        } else {
            let r = (origin.y.value - origin2.y.value) / direction2.y.value;
            Point::new(
                origin2.x.value + direction2.x.value * r,
                origin2.y.value + direction2.y.value * r,
            )
        }
    }
    fn find_possible_cord_intersecting_edges<'a>(
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
    fn create_cords(&self) -> Vec<Cord> {
        let mut cords = vec![];
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
                self.find_possible_cord_intersecting_edges(vertex, i, right_vertices.clone());
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
                self.find_possible_cord_intersecting_edges(vertex, i, left_vertices.clone());
            // println!("Left edges: {:?}", left_edges);

            let (left_edges, right_edges, wrapped_right, wrapped_left) =
                if right_edges.is_empty() && !left_edges.is_empty() {
                    // println!("No right edges intersecting with vertex {i} cords, coming from the left");
                    let wrap_around_vertex = Point::new(FixedNumber(-1 * SCALE), vertex.y.value);
                    let right_edges = self.find_possible_cord_intersecting_edges(
                        &wrap_around_vertex,
                        i,
                        left_vertices,
                    );
                    // println!("Found Right edges: {:?}", right_edges);
                    (left_edges, right_edges, true, false)
                } else if left_edges.is_empty() && !right_edges.is_empty() {
                    // println!("No left edges intersecting with vertex {i} cords, coming from the right");
                    let wrap_around_vertex = Point::new(FixedNumber(-1 * SCALE), vertex.y.value);
                    let left_edges = self.find_possible_cord_intersecting_edges(
                        &wrap_around_vertex,
                        i,
                        right_vertices,
                    );
                    // println!("Found Left edges: {:?}", left_edges);
                    (left_edges, right_edges, true, false)
                } else {
                    (left_edges, right_edges, false, false)
                };
            if right_edges.is_empty() && left_edges.is_empty() {
                // println!("No edges intersecting with vertex {i} cords, intersecting with it self");
                cords.push(Cord {
                    origin_vertex: i,
                    intersection: None,
                    wrap_around: true,
                });
                continue;
            }

            // println!("Right edges: {:?}", right_edges);
            let right_intersection = right_edges
                .iter()
                .filter_map(|e| {
                    // println!("edge {e}: {:?}", self.edges[*e]);
                    let intersection = self.cord_edge_intersection(i, &self.edges[*e]);
                    // println!("Intersection: {:?}", intersection);
                    if wrapped_right || intersection.x.value > vertex.x.value {
                        Some((e, intersection))
                    } else {
                        None
                    }
                })
                .fold(
                    (0_usize, Point::new(FixedNumber(i128::MAX), FixedNumber(0))),
                    |(e_min, min_i), (e, i)| {
                        if i.x.value < min_i.x.value {
                            (*e, i)
                        } else {
                            (e_min, min_i)
                        }
                    },
                );
            // println!("Right Intersection: {:?}", right_intersection);
            cords.push(Cord {
                origin_vertex: i,
                intersection: Some(right_intersection),
                wrap_around: wrapped_right,
            });

            // println!("Left edges: {:?}", right_edges);
            let left_intersection = left_edges
                .iter()
                .filter_map(|e| {
                    // println!("edge {e}: {:?}", self.edges[*e]);
                    let intersection = self.cord_edge_intersection(i, &self.edges[*e]);
                    // println!("Intersection: {:?}", intersection);
                    if wrapped_left || intersection.x.value < vertex.x.value {
                        Some((e, intersection))
                    } else {
                        None
                    }
                })
                .fold(
                    (0_usize, Point::new(FixedNumber(i128::MIN), FixedNumber(0))),
                    |(e_max, max_i), (e, i)| {
                        if i.x.value > max_i.x.value {
                            (*e, i)
                        } else {
                            (e_max, max_i)
                        }
                    },
                );
            // println!("Left Intersection: {:?}", left_intersection);
            cords.push(Cord {
                origin_vertex: i,
                intersection: Some(left_intersection),
                wrap_around: wrapped_left,
            });
        }
        cords
    }
    fn order_chords_and_vertices(&self) -> Vec<DoubleBoundaryPoint> {
        let mut points = vec![];
        let mut inverse_points = vec![];
        for (i, edge) in self.edges.iter().enumerate() {
            // println!("\n------------------------------");
            // println!("edge: {:?}", i);
            points.push(DoubleBoundaryPoint::Vertex(edge.start));
            inverse_points.push(DoubleBoundaryPoint::Vertex(edge.start));
            let (mut left, mut right): (Vec<_>, Vec<_>) = self
                .cords
                .iter()
                .enumerate()
                .filter_map(|(cord_i, c)| {
                    if let Some((e, _)) = &c.intersection {
                        if e == &i {
                            Some((cord_i, c))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .partition(|(cord_i, c)| {
                    let origin = if c.wrap_around {
                        Point::new(
                            FixedNumber(
                                if self.vertices[c.origin_vertex].x.value
                                    > c.intersection.as_ref().unwrap().1.x.value
                                {
                                    println!("WRAP: -1");
                                    -1
                                } else {
                                    println!("WRAP: 1");
                                    1
                                } * SCALE,
                            ),
                            self.vertices[c.origin_vertex].y.value,
                        )
                    } else {
                        Point::new(
                            self.vertices[c.origin_vertex].x.value,
                            self.vertices[c.origin_vertex].y.value,
                        )
                    };
                    let edge_start = &self.vertices[edge.start];
                    let direction = Point::new(
                        self.vertices[edge.end].x.value - edge_start.x.value,
                        self.vertices[edge.end].y.value - edge_start.y.value,
                    );
                    // rotate direction by 270 degrees (rd) and find the intersection between origin + t * rd and an infinitly extended edge
                    // if t is greater than 0 the intersection is on the left side of the edge
                    let t = if direction.x.value.0 == 0 {
                        (edge_start.x.value - origin.x.value) / direction.y.value
                    } else if direction.y.value.0 == 0 {
                        (origin.y.value - edge_start.y.value) / direction.x.value
                    } else {
                        let v = (edge_start.x.value - origin.x.value) / direction.y.value
                            + (origin.y.value - edge_start.y.value) * direction.x.value
                                / (direction.y.value * direction.y.value);
                        v / (FixedNumber(SCALE)
                            + direction.x.value * direction.x.value
                                / (direction.y.value * direction.y.value))
                    };

                    // println!("cord_i: {:?}", cord_i);
                    // println!("t: {:?}", t);

                    t.0 > 0
                });

            left.sort_by(|(_, a), (_, b)| {
                let a = &a.intersection.as_ref().unwrap().1;
                let b = &b.intersection.as_ref().unwrap().1;
                let distance_a = (a.x.value - self.vertices[edge.start].x.value).0.pow(2)
                    + (a.y.value - self.vertices[edge.start].y.value).0.pow(2);
                let distance_b = (b.x.value - self.vertices[edge.start].x.value).0.pow(2)
                    + (b.y.value - self.vertices[edge.start].y.value).0.pow(2);
                distance_a.partial_cmp(&distance_b).unwrap()
            });
            for (cord_i, _) in left {
                points.push(DoubleBoundaryPoint::CordEndpoint(cord_i));
            }
            right.sort_by(|(_, a), (_, b)| {
                let a = &a.intersection.as_ref().unwrap().1;
                let b = &b.intersection.as_ref().unwrap().1;
                let distance_a = (a.x.value - self.vertices[edge.start].x.value).0.pow(2)
                    + (a.y.value - self.vertices[edge.start].y.value).0.pow(2);
                let distance_b = (b.x.value - self.vertices[edge.start].x.value).0.pow(2)
                    + (b.y.value - self.vertices[edge.start].y.value).0.pow(2);
                distance_a.partial_cmp(&distance_b).unwrap()
            });
            for (cord_i, _) in right {
                inverse_points.push(DoubleBoundaryPoint::CordEndpoint(cord_i));
            }
        }
        points.push(DoubleBoundaryPoint::Vertex(self.vertices.len() - 1));
        points.extend(inverse_points.into_iter().rev());
        points
    }
    fn create_visibility_map(&mut self) {
        let cords = self.create_cords();
        // println!("Cords: {:#?}", cords);
        self.cords = cords;
        let chanonical_vertex_enumeration = self.order_chords_and_vertices();
        println!("All Points: {:#?}", chanonical_vertex_enumeration);
    }
    fn triangulate(&mut self) {
        self.create_visibility_map();
        self.triangles.clear();
        // for i in 1..self.vertices.len() - 1 {
        //     self.triangles.push([0, i, i + 1]);
        // }
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
            .chain(
                self.cords
                    .iter()
                    .map(|cord| {
                        let width = 0.002;
                        let endpoint = match &cord.intersection {
                            Some((_, p)) => p,
                            None => &self.vertices[cord.origin_vertex],
                        };
                        let inf = Point::new(1.0.into(), self.vertices[cord.origin_vertex].y.value);
                        let minus_inf = Point::new((-1.0).into(), inf.y.value);
                        let pieces = if cord.wrap_around {
                            let (out_point, in_point) =
                                if endpoint.x.value < self.vertices[cord.origin_vertex].x.value {
                                    (&inf, &minus_inf)
                                } else {
                                    (&minus_inf, &inf)
                                };
                            vec![
                                (&self.vertices[cord.origin_vertex], out_point),
                                (in_point, endpoint),
                            ]
                        } else {
                            vec![(&self.vertices[cord.origin_vertex], endpoint)]
                        };
                        pieces
                            .into_iter()
                            .map(|(start, end)| {
                                [
                                    (
                                        [
                                            Vector::new(
                                                f32::from(start.x.value),
                                                f32::from(start.y.value) + width,
                                                0.0,
                                            ),
                                            Vector::new(
                                                f32::from(start.x.value),
                                                f32::from(start.y.value) - width,
                                                0.0,
                                            ),
                                            Vector::new(
                                                f32::from(end.x.value),
                                                f32::from(end.y.value) - width,
                                                0.0,
                                            ),
                                        ],
                                        Color::from_str("black"),
                                    ),
                                    (
                                        [
                                            Vector::new(
                                                f32::from(end.x.value),
                                                f32::from(end.y.value) - width,
                                                0.0,
                                            ),
                                            Vector::new(
                                                f32::from(end.x.value),
                                                f32::from(end.y.value) + width,
                                                0.0,
                                            ),
                                            Vector::new(
                                                f32::from(start.x.value),
                                                f32::from(start.y.value) + width,
                                                0.0,
                                            ),
                                        ],
                                        Color::from_str("black"),
                                    ),
                                ]
                            })
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .flatten(),
            )
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
