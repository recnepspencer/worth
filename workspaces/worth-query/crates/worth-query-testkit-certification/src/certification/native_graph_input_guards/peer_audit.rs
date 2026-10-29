//! Retained parent audit has a separate identity and lifetime from a disposable selection.
#[path = "peer_audit/actions.rs"]
pub mod actions;
#[path = "peer_audit/query.rs"]
pub mod query;
#[path = "peer_audit/schema.rs"]
pub mod schema;
pub use actions::{Append, Binding};
pub use query::{Read, ReadBinding, Value};
pub use schema::*;
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<
        crate::facade::certification::native_graph_input_guards::schema::Schema,
    >,
) -> ApplicationSchemaDeclarationBuilder<
    crate::facade::certification::native_graph_input_guards::schema::Schema,
> {
    actions::declare(schema::declare(builder))
        .application_query(query::definition())
        .application_query_binding::<ReadBinding>()
}
