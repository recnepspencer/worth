//! Select the maintained record's existing numerical field and binding.
use super::super::schema::{CatalogRecords, Item, ItemFacts};
use super::coordinates::Coordinates;
use worth_query_decl::facade::application_schema::fragment::FieldSelection;

// CatalogRecordsBindings supplies CoordinatesBinding, including Box2's actual
// unit and frame. A second unit marker cannot redefine that carried contract.
pub type ItemBounds = FieldSelection<CatalogRecords, Item, ItemFacts, Coordinates>;
