use std::fmt::{Debug, Display};

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
        let _left: Option<Box<dyn Edge>> = None;
        let _up: Option<Box<dyn Edge>> = None;
        let _right: Option<Box<dyn Edge>> = None;
        let _down: Option<Box<dyn Edge>> = None;
        let _occupier: Option<Box<dyn Center>> = None;
        $(
            match $direction {
                "l" | "left" | "L" | "Left" | "LEFT" => {
                    let _left: Option<Box<dyn Edge>> = Some(Box::new($obstacle));
                },
                "u" | "up" | "U" | "Up" | "UP" => {
                    let _up: Option<Box<dyn Edge>> = Some(Box::new($obstacle));
                },
                "r" | "right" | "R" | "Right" | "RIGHT" => {
                    let _right: Option<Box<dyn Edge>> = Some(Box::new($obstacle));
                },
                "d" | "down" | "D" | "Down" | "DOWN" => {
                    let _down: Option<Box<dyn Edge>> = Some(Box::new($obstacle));
                },
                _ => {},
            }
        )*
        $(
            let _occupier: Option<Box<dyn Center>> = Some(Box::new($occupier));
        )?
        Tile{left: _left, up: _up, right: _right, down: _down, occupier: _occupier}
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
            let length = if left_edge { 1 } else { 2 };
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
            let length = if left_edge { 1 } else { 2 };
            let downstacle = downstacle.as_char(Direction::Down);
            for _ in 0..length {
                down.push(downstacle);
            }
            downstacle
        } else {
            down += if left_edge { " " } else { "  " };
            ' '
        };
        if let Some(occupier) = &self.occupier {
            middle.push(occupier.as_char());
        } else {
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
enum DungeonLayoutBuilderErr {
    UnhandledSymbol(char),
    MalformedLayout(String),
    EmptyDungeon,
}
impl Debug for DungeonLayoutBuilderErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DungeonLayoutBuilderErr::UnhandledSymbol(x) => format!("Unhandled Symbol: {}", x),
            DungeonLayoutBuilderErr::MalformedLayout(x) => format!("Malformed Layout: {}", x),
            DungeonLayoutBuilderErr::EmptyDungeon => "Provided dungeon is empty".to_string(),
        };
        write!(f, "{}", s)
    }
}
type BuilderRes<T> = Result<T, DungeonLayoutBuilderErr>;
type EdgeFactory = Box<dyn Fn(char, Direction) -> BuilderRes<Option<Box<dyn Edge>>>>;
type CenterFactory = Box<dyn Fn(char) -> BuilderRes<Option<Box<dyn Center>>>>;
#[derive(Default)]
/// If a factory returns `DungeonLayoutBuilderErr::UnhandledSymbol` the default factory will handle the
/// symbol.
pub struct DungeonLayoutBuilder {
    edge_factory: Option<EdgeFactory>,
    center_factory: Option<CenterFactory>,
}
impl DungeonLayoutBuilder {
    pub fn with_edge_factory(mut self, factory: Box<EdgeFactory>) -> Self {
        self.edge_factory = Some(factory);
        self
    }
    pub fn with_center_factory(mut self, factory: Box<CenterFactory>) -> Self {
        self.center_factory = Some(factory);
        self
    }
    fn default_edge_factory(
        &self,
        symbol: char,
        direction: Direction,
    ) -> BuilderRes<Option<Box<dyn Edge>>> {
        match symbol {
            ' ' => Ok(None),
            'T' => Ok(Some(Box::new(Trap))),
            x => Err(DungeonLayoutBuilderErr::UnhandledSymbol(x)),
        }
    }
    fn default_center_factory(&self, symbol: char) -> BuilderRes<Option<Box<dyn Center>>> {
        match symbol {
            ' ' => Ok(None),
            'T' => Ok(Some(Box::new(Trap))),
            x => Err(DungeonLayoutBuilderErr::UnhandledSymbol(x)),
        }
    }
    pub fn build(&self, text: &str) -> Result<DungeonLayout, DungeonLayoutBuilderErr> {
        let lines = text.lines().collect::<Vec<_>>();
        let width = match lines.first() {
            None => return Err(DungeonLayoutBuilderErr::EmptyDungeon),
            Some(l) => l.len(),
        };
        if width % TILESIZE != 0 {
            return Err(DungeonLayoutBuilderErr::MalformedLayout(format!(
                "Expected tiles to be {} chars wide, but encountered {} chars in the first line",
                TILESIZE, width
            )));
        }
        let tile_lines = lines.chunks_exact(TILESIZE);
        let remainder = tile_lines.remainder();
        if !remainder.is_empty() {
            return Err(DungeonLayoutBuilderErr::MalformedLayout(format!(
                "Expected tiles to be {} lines high, but the last one is only {}:\n{:#?}",
                TILESIZE,
                remainder.len(),
                remainder
            )));
        }
        let mut tiles = vec![];
        for (y, tile_parts) in tile_lines.enumerate() {
            let up = tile_parts[0];
            let middle = tile_parts[1];
            let down = tile_parts[2];
            if up.len() != middle.len() || up.len() != down.len() {
                return Err(DungeonLayoutBuilderErr::MalformedLayout(format!("Expected tiles to be {} lines of the same length, but encountered different lengths in {}th tile-line", TILESIZE, y)));
            }
            if up.len() != width {
                return Err(DungeonLayoutBuilderErr::MalformedLayout(format!("Expected all tile-lines to have the same width of {} chars but encountered only {} chars in {}th tile-line", width, up.len(), y)));
            }
            let up = up.chars().skip(1).step_by(3);
            let left = middle.chars().step_by(3);
            let center = middle.chars().skip(1).step_by(3);
            let right = middle.chars().skip(2).step_by(3);
            let down = down.chars().skip(1).step_by(3);
            for (up, (left, (center, (right, down)))) in
                up.zip(left.zip(center.zip(right.zip(down))))
            {
                let up = self.default_edge_factory(up, Direction::Up)?;
                let left = self.default_edge_factory(left, Direction::Left)?;
                let right = self.default_edge_factory(right, Direction::Right)?;
                let down = self.default_edge_factory(down, Direction::Down)?;

                let center = self.default_center_factory(center)?;
                tiles.push(Tile {
                    up,
                    left,
                    right,
                    down,
                    occupier: center,
                });
            }
        }
        Ok(DungeonLayout {
            tiles,
            width: (width / TILESIZE) as u8,
        })
    }
}
impl Debug for DungeonLayout {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut up = String::with_capacity((TILESIZE as u8 * self.width).into());
        let mut middle = String::with_capacity((TILESIZE as u8 * self.width).into());
        let mut down = String::with_capacity((TILESIZE as u8 * self.width).into());
        let mut x = 0;
        for tile in &self.tiles {
            if x >= self.width {
                x = 0;
                up = format!("{up}\n{middle}\n{down}\n");
                middle = String::with_capacity((TILESIZE as u8 * self.width).into());
                down = String::with_capacity((TILESIZE as u8 * self.width).into());
            }
            let tile = tile.to_string();
            let (tile_up, tile) = tile.split_once("\n").expect(&format!(
                "Expected Tile to have at least 2 lines, but got '{:?}'",
                tile
            ));
            let (tile_middle, tile_down) = tile.split_once("\n").expect(&format!(
                "Expected Tile to have {} lines, but got '{:?}\n{:?}'",
                TILESIZE, tile_up, tile
            ));
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
        assert!(
            example_string
                == "\
|--|-|
|  |T|
|  |T|
--|  |
  | T|
  |--|"
        );
        let reverse: DungeonLayout = DungeonLayoutBuilder::default()
            .build(&example_string)
            .unwrap();
        assert!(format!("{:?}", reverse) == example_string);
    }
}
