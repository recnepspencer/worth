//! Checked closed bounds in one exact coordinate/unit/frame domain.
use crate::{BoxSpace, SignedFixture};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Canonical input coordinates; validation belongs to the numerical constructor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Box2Coordinates<C = i64> {
    pub lower: [C; 2],
    pub upper: [C; 2],
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
/// Domain class does not establish affinity between different model instances.
///
/// ```
/// use worth_query_consumer_values::{IntegerBox2, ModelLocalMetre, Box2Coordinates};
/// let a = IntegerBox2::<ModelLocalMetre>::new(Box2Coordinates { lower: [0;2], upper: [u64::MAX;2] }).unwrap();
/// assert!(a.overlaps(a.union(a)));
/// ```
/// ```compile_fail
/// use worth_query_consumer_values::{Box2, IntegerBox2, ModelLocalMetre, Box2Coordinates};
/// let signed = Box2::new(Box2Coordinates { lower: [0;2], upper: [1;2] }).unwrap();
/// let metres = IntegerBox2::<ModelLocalMetre>::new(Box2Coordinates { lower: [0;2], upper: [1;2] }).unwrap();
/// signed.overlaps(metres);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegerBox2<Space: BoxSpace>(Box2Coordinates<Space::Coordinate>, PhantomData<Space>);

/// The existing signed fixture domain, sharing the sole numerical implementation.
pub type Box2 = IntegerBox2<SignedFixture>;

impl<Space: BoxSpace> IntegerBox2<Space> {
    pub const LENGTH_UNIT: &'static str = Space::LENGTH_UNIT;
    pub const FRAME: &'static str = Space::FRAME;

    pub fn new(coordinates: Box2Coordinates<Space::Coordinate>) -> Result<Self, BoundsDenial> {
        for axis in 0..2 {
            if coordinates.lower[axis] > coordinates.upper[axis] {
                return Err(BoundsDenial::ReversedAxis { axis: axis as u8 });
            }
        }
        Ok(Self(coordinates, PhantomData))
    }

    pub const fn coordinates(self) -> Box2Coordinates<Space::Coordinate> {
        self.0
    }

    /// Exact enclosing bounds: no conversion, expansion, rounding or epsilon.
    pub fn union(self, other: Self) -> Self {
        Self(
            Box2Coordinates {
                lower: [
                    self.0.lower[0].min(other.0.lower[0]),
                    self.0.lower[1].min(other.0.lower[1]),
                ],
                upper: [
                    self.0.upper[0].max(other.0.upper[0]),
                    self.0.upper[1].max(other.0.upper[1]),
                ],
            },
            PhantomData,
        )
    }

    /// Inclusive overlap includes a shared boundary point or edge.
    pub fn overlaps(self, other: Self) -> bool {
        (0..2).all(|axis| {
            self.0.lower[axis] <= other.0.upper[axis] && other.0.lower[axis] <= self.0.upper[axis]
        })
    }
}
