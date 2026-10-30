//! The same required numerical codec/unit under this genuine application schema.
#[path = "../boxes/coordinates.rs"]
mod coordinates;
#[path = "../boxes/field.rs"]
mod field;
pub(super) use coordinates::{Binding as CoordinatesBinding, Coordinates};
pub(super) use field::{FixtureLength, ItemBounds};
