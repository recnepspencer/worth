//! The two exact coordinate/unit/frame domains used by these numerical owners.
use crate::PlanarInteger;
mod sealed {
    pub trait Space {}
}
/// A closed domain, not a runtime coordinate conversion or geometric authority.
pub trait BoxSpace: sealed::Space + Copy + std::fmt::Debug + Eq {
    type Coordinate: PlanarInteger + Ord + std::fmt::Debug;
    const LENGTH_UNIT: &'static str;
    const FRAME: &'static str;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SignedFixture;
impl sealed::Space for SignedFixture {}
impl BoxSpace for SignedFixture {
    type Coordinate = i64;
    const LENGTH_UNIT: &'static str = "fixture-integer-length-v1";
    const FRAME: &'static str = "fixture.cartesian-xy.v1";
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelLocalMetre;
impl sealed::Space for ModelLocalMetre {}
impl BoxSpace for ModelLocalMetre {
    type Coordinate = u64;
    const LENGTH_UNIT: &'static str = "Metre";
    const FRAME: &'static str = "worth.frames.model-local.v1";
}
