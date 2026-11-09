//! A child-owned auxiliary entity, never included in the parent's output witness.
use super::*;

pub type CellTarget = WorthQueryInvariantMutationTarget<Schema, ProducerCell>;
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
}

pub use super::schema::storage_model::{CellKey, CellValue, ProducerCell};
