//! Retained parent audit has a separate identity and lifetime from a disposable selection.
#[path = "peer_audit/actions.rs"]
mod actions;
#[path = "peer_audit/query.rs"]
mod query;
#[path = "peer_audit/schema.rs"]
mod schema;
pub use actions::{Append, Binding};
pub use query::{Read, ReadBinding, Value};
pub use schema::*;
use worth_query_decl::facade::application_schema::*;
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<super::schema::Schema>,
) -> ApplicationSchemaDeclarationBuilder<super::schema::Schema> {
    actions::declare(schema::declare(builder))
        .application_query(query::definition())
        .application_query_binding::<ReadBinding>()
}
