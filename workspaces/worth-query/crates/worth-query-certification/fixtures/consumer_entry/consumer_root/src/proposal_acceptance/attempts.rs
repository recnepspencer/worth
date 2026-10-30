//! Stable ordinary registration and descriptive completion records for one model.
pub mod claim;
pub mod data;
pub mod query;
pub mod recording;
pub mod result_query;
pub mod schema;
use super::schema::Schema;
use worth_query_decl::facade::application_schema::*;
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    recording::declare(claim::declare(result_query::declare(query::declare(
        schema::declare(builder),
    ))))
}
