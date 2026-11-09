//! The Candidate record selects the existing coordinate codec's unit/frame.
use super::super::schema::{CatalogRecords, Item, ItemFacts};
use super::coordinates::Coordinates;
use worth_query_decl::facade::application_schema::fragment::FieldSelection;
pub type ItemBounds = FieldSelection<CatalogRecords, Item, ItemFacts, Coordinates>;
