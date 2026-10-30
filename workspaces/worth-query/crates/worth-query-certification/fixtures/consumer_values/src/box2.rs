//! Closed integer boxes in this fixture's one explicit length unit and XY frame.
use serde::{Deserialize, Serialize};

/// Canonical input coordinates; validation belongs to the numerical constructor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Box2Coordinates {
    pub lower: [i64; 2],
    pub upper: [i64; 2],
}

/// A reversed interval is not a valid closed box.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundsDenial {
    ReversedAxis { axis: u8 },
}
impl std::fmt::Display for BoundsDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReversedAxis { axis } => {
                write!(formatter, "reversed closed interval on axis {axis}")
            }
        }
    }
}
impl std::error::Error for BoundsDenial {}

/// Validated finite bounds. All coordinates use the same declared unit/frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Box2(Box2Coordinates);

impl Box2 {
    pub const LENGTH_UNIT: &'static str = "fixture-integer-length-v1";
    pub const FRAME: &'static str = "fixture.cartesian-xy.v1";

    pub fn new(coordinates: Box2Coordinates) -> Result<Self, BoundsDenial> {
        for axis in 0..2 {
            if coordinates.lower[axis] > coordinates.upper[axis] {
                return Err(BoundsDenial::ReversedAxis { axis: axis as u8 });
            }
        }
        Ok(Self(coordinates))
    }

    pub const fn coordinates(self) -> Box2Coordinates {
        self.0
    }

    /// Exact enclosing bounds: no conversion, expansion, rounding or epsilon.
    pub fn union(self, other: Self) -> Self {
        Self(Box2Coordinates {
            lower: [
                self.0.lower[0].min(other.0.lower[0]),
                self.0.lower[1].min(other.0.lower[1]),
            ],
            upper: [
                self.0.upper[0].max(other.0.upper[0]),
                self.0.upper[1].max(other.0.upper[1]),
            ],
        })
    }

    /// Inclusive overlap includes a shared boundary point or edge.
    pub fn overlaps(self, other: Self) -> bool {
        (0..2).all(|axis| {
            self.0.lower[axis] <= other.0.upper[axis] && other.0.lower[axis] <= self.0.upper[axis]
        })
    }
}
