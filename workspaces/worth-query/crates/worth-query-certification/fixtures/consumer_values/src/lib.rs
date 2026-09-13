#![forbid(unsafe_code)]

mod planar_operation;
mod planar_replacement;
pub use planar_replacement::{
    PlanarReplacementDenial, PlanarVertexReplacement, PlanarVertexReplacementResult,
};

pub use planar_operation::{
    PlanarAdjustment, PlanarAdjustmentResult, PlanarMutationDenial, PlanarOperation, PlanarVertex,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PositiveLength(u64);

impl PositiveLength {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn get(value: &Self) -> u64 {
        value.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PositiveCount(u64);

impl PositiveCount {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn get(value: &Self) -> u64 {
        value.0
    }
}
