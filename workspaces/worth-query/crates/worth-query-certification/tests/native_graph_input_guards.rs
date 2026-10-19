//! Native path-input boundaries, without compiling unrelated workflow portfolios.
#[path = "native_graph_input_guards/actions.rs"]
mod actions;
#[path = "native_graph_input_guards/admission.rs"]
mod admission;
#[path = "native_graph_input_guards/audit_query.rs"]
mod audit_query;
#[path = "native_graph_input_guards/evidence.rs"]
mod evidence;
#[path = "application_graph/document_retention_model/operator_identity.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "Reuse real authentication ports; reviewer variants are unrelated to this boundary."
)]
mod operator_identity;
#[path = "native_graph_input_guards/program.rs"]
mod program;
#[path = "native_graph_input_guards/row_query.rs"]
mod row_query;
#[path = "native_graph_input_guards/schema.rs"]
mod schema;
#[path = "native_graph_input_guards/world.rs"]
mod world;
#[path = "application_graph/document_retention_model/host/world_resources.rs"]
#[allow(
    dead_code,
    reason = "Reuse the ordinary bounded World policy, not its separate scale profiles."
)]
mod world_resources;
