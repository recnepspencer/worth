//! The typed Source field uses the exact numerical representation unit.
use super::super::schema::{Item, ItemFacts};
use super::coordinates::Coordinates;
pub type ItemBounds = worth_query_decl::facade::application_schema::fragment::FieldSelection<
    super::super::schema::CatalogRecords,
    Item,
    ItemFacts,
    Coordinates,
>;
