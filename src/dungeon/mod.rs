use std::fmt::{Debug, Display};
use std::str::FromStr;

trait Edge {
    fn as_char(&self, direction: Direction) -> char;
}

trait Center {
    fn as_char(&self) -> char;
}

trait Occupance {
    fn as_char(&self, direction: Option<Direction>) -> char;
}
impl<O: Occupance> Edge for O {
    fn as_char(&self, direction: Direction) -> char {
        Occupance::as_char(self, Some(direction))
    }
}
impl<O: Occupance> Center for O {
    fn as_char(&self) -> char {
        Occupance::as_char(self, None)
    }
}

struct Wall;
impl Edge for Wall {
    fn as_char(&self, direction: Direction) -> char {
        match direction {
            Direction::Left | Direction::Right => '|',
            Direction::Up | Direction::Down => '-',
        }
    }
}

struct Trap;
impl Occupance for Trap {
    fn as_char(&self, direction: Option<Direction>) -> char {
        'T'
    }
}

enum Direction {
    Left,
    Up,
    Right,
    Down,
}
struct Tile {
    left: Option<Box<dyn Edge>>,
    up: Option<Box<dyn Edge>>,
    right: Option<Box<dyn Edge>>,
    down: Option<Box<dyn Edge>>,
    occupier: Option<Box<dyn Center>>,
}
macro_rules! tile {
    [$($direction:literal: $obstacle:expr),*$(; $occupier:expr)?] => {{
        let mut left: Option<Box<dyn Edge>> = None;
        let mut up: Option<Box<dyn Edge>> = None;
        let mut right: Option<Box<dyn Edge>> = None;
        let mut down: Option<Box<dyn Edge>> = None;
        let mut occupier: Option<Box<dyn Center>> = None;
        $(
            match $direction {
                "l" | "left" | "L" | "Left" | "LEFT" => {
                    left = Some(Box::new($obstacle));
                },
                "u" | "up" | "U" | "Up" | "UP" => {
                    up = Some(Box::new($obstacle));
                },
                "r" | "right" | "R" | "Right" | "RIGHT" => {
                    right = Some(Box::new($obstacle));
                },
                "d" | "down" | "D" | "Down" | "DOWN" => {
                    down = Some(Box::new($obstacle));
                },
                _ => {},
            }
        )*
        $(
            occupier = Some(Box::new($occupier));
        )?
        Tile{left, up, right, down, occupier}
    }}
}
const TILESIZE: usize = 3;
impl Display for Tile {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut up = String::with_capacity(TILESIZE);
        let mut middle = String::with_capacity(TILESIZE);
        let mut down = String::with_capacity(TILESIZE);
        let left_edge = if let Some(left) = &self.left {
            let left = left.as_char(Direction::Left);
            up.push(left);
            middle.push(left);
            down.push(left);
            true
        } else { 
            middle += " ";
            false
        };
        let up_edge = if let Some(upstacle) = &self.up {
            let length = if left_edge {
                1
            } else {
                2
            };
            let upstacle = upstacle.as_char(Direction::Up);
            for _ in 0..length {
                up.push(upstacle);
            }
            upstacle
        } else {
            up += if left_edge { " " } else { "  " };
            ' ' 
        };
        let down_edge = if let Some(downstacle) = &self.down {
            let length = if left_edge {
                1
            } else {
                2
            };
            let downstacle = downstacle.as_char(Direction::Down);
            for _ in 0..length {
                down.push(downstacle);
            }
            downstacle
        } else {
            down += if left_edge { " " } else { "  " };
            ' '
        };
        let occupier = if let Some(occupier) = &self.occupier {
            middle.push(occupier.as_char());
        }   else {
            middle += " ";
        };
        if let Some(right) = &self.right {
            let right = right.as_char(Direction::Right);
            up.push(right);
            middle.push(right);
            down.push(right);
        } else {
            up.push(up_edge);
            middle += " ";
            down.push(down_edge);
        }
        write!(w, "{up}\n{middle}\n{down}")
    }
}

/// |-------|
/// |   |   |
/// |       |
/// |---|   |
///     |   |
///     |   |
/// |---|   |
/// |       |
/// |       |
/// |-------|

struct DungeonLayout {
    tiles: Vec<Tile>,
    width: u8,
}
impl FromStr for DungeonLayout {
    type Err = Box<dyn std::error::Error>;
    fn from_str(layout: &str) -> Result<Self, <Self as FromStr>::Err> { 
        let lines = layout.split("\n");
        todo!("Builder with Factory(occupanceFactory) to keep moddability the traits garant");
    }
}
impl Debug for DungeonLayout {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut up = String::with_capacity((TILESIZE as u8 * self.width).into());
        let mut middle = String::with_capacity((TILESIZE as u8 * self.width).into());
        let mut down = String::with_capacity((TILESIZE as u8 * self.width).into());
        let mut y = 0;
        let mut x = 0;
        for tile in &self.tiles {
            if x >= self.width {
                y += 1;
                x = 0;
                up = format!("{up}\n{middle}\n{down}\n");
                middle = String::with_capacity((TILESIZE as u8 * self.width).into());
                down = String::with_capacity((TILESIZE as u8 * self.width).into());
            }
            let tile = tile.to_string();
            let (tile_up, tile) = tile
                .split_once("\n")
                .expect(&format!("Expected Tile to have at least 2 lines, but got '{:?}'", tile));
            let (tile_middle, tile_down) = tile
                .split_once("\n")
                .expect(&format!("Expected Tile to have {} lines, but got '{:?}\n{:?}'", TILESIZE, tile_up, tile));
            up += tile_up;
            middle += tile_middle;
            down += tile_down;
            x += 1;
        }
        write!(w, "{up}\n{middle}\n{down}")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build() {
        let example = DungeonLayout {
            tiles: vec![
                tile!["l": Wall, "u": Wall],
                tile!["u": Wall, "r": Wall, "l": Wall, "d": Trap; Trap],
                tile!["u": Wall, "r": Wall],
                tile!["r": Wall, "d": Wall; Trap],
            ],
            width: 2,
        };
        let example_string = format!("{:?}", example);
        println!("{}", example_string);
        assert!(example_string == "\
|--|-|
|  |T|
|  |T|
--|  |
  | T|
  |--|");
        let reverse: DungeonLayout = example_string.parse().unwrap();
        assert!(format!("{:?}", reverse) == example_string);
    }
}
