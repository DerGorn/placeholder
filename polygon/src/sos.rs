use std::ops::{Add, Div, Mul, Sub};
#[derive(Clone, Copy, PartialEq, Default, PartialOrd)]
pub struct FixedNumber(i128);
impl std::fmt::Debug for FixedNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "F({})", f32::from(*self))
    }
}
pub const SCALE: i128 = 2 << 61;
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
impl From<i128> for FixedNumber {
    fn from(i: i128) -> Self {
        Self(i * SCALE)
    }
}
impl PartialEq<i128> for FixedNumber {
    fn eq(&self, other: &i128) -> bool {
        self.0 == *other
    }
}
impl PartialOrd<i128> for FixedNumber {
    fn partial_cmp(&self, other: &i128) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

pub struct Coordinate {
    pub value: FixedNumber,
    index: usize,
}
impl Coordinate {
    pub fn smaller(&self, self_point: usize, other: &Self, other_point: usize) -> bool {
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
    pub x: Coordinate,
    pub y: Coordinate,
}
impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {:?})", self.x.value, self.y.value)
    }
}
impl Point {
    pub fn x(&self) -> i128 {
        self.x.value.0
    }
    pub fn y(&self) -> i128 {
        self.y.value.0
    }
    pub fn new(x: FixedNumber, y: FixedNumber) -> Self {
        Self {
            x: Coordinate { value: x, index: 0 },
            y: Coordinate { value: y, index: 1 },
        }
    }
    pub fn from_i128(x: i128, y: i128) -> Self {
        Self {
            x: Coordinate {
                value: FixedNumber(x),
                index: 0,
            },
            y: Coordinate {
                value: FixedNumber(y),
                index: 1,
            },
        }
    }
    pub fn from_f32(x: f32, y: f32) -> Self {
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
