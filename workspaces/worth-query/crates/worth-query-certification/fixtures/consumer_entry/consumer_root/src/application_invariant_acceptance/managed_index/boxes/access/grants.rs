//! Real grant facts and scope relations consumed by the existing capability owner.
mod storage_model {
    include!("grants/records.rs");
}
use super::super::super::schema::*;
pub use storage_model::*;
use worth_query_decl::facade::application_schema::*;

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    schema
}
