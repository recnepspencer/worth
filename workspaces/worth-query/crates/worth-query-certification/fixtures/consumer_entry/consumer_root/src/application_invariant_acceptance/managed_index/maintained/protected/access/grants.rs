//! Schema-bound fixture for managed_index::maintained::protected::access.
//! Real grant facts and scope relations consumed by the existing capability owner.
#[path = "grants/records.rs"]
mod storage_model;
use super::super::super::schema::*;
pub use storage_model::*;
use worth_query_decl::facade::application_schema::*;

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    schema
}
