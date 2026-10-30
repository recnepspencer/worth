//! The typed Source field uses the exact numerical representation unit.
use super::super::schema::{Catalog, Item, ItemFacts};
use super::coordinates::{Binding, Coordinates};
use worth_query_consumer_values::Box2;
use worth_query_decl::facade::{application_schema::*, worth_query_field};
pub struct FixtureLength;
impl ApplicationUnitMarker<()> for FixtureLength {
    const NAME: &'static str = Box2::LENGTH_UNIT;
}
impl FixtureLength {
    pub const fn reference() -> ApplicationUnitRef<Catalog, Self> {
        ApplicationUnitRef::from_schema_identifier(Box2::LENGTH_UNIT)
    }
}
worth_query_field!(pub ItemBounds for Catalog, Item, ItemFacts: Coordinates => Binding, unit FixtureLength, read_only, equality);
