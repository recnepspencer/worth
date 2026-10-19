//! Native path-input boundaries, without compiling unrelated workflow portfolios.
#[path = "native_graph_input_guards/actions.rs"]
mod actions;
#[path = "native_graph_input_guards/admission.rs"]
mod admission;
#[path = "native_graph_input_guards/audit_query.rs"]
mod audit_query;
#[path = "native_graph_input_guards/child_actions.rs"]
mod child_actions;
#[path = "native_graph_input_guards/child_admission.rs"]
mod child_admission;
#[path = "native_graph_input_guards/child_evidence.rs"]
mod child_evidence;
#[path = "native_graph_input_guards/child_query.rs"]
mod child_query;
#[path = "native_graph_input_guards/current_snapshot_query.rs"]
mod current_snapshot_query;
#[path = "native_graph_input_guards/evidence.rs"]
mod evidence;
#[path = "application_graph/document_retention_model/operator_identity.rs"]
#[allow(
    dead_code,
    unused_imports,
    reason = "Reuse real authentication ports; reviewer variants are unrelated to this boundary."
)]
mod operator_identity;
#[path = "native_graph_input_guards/path_signing.rs"]
mod path_signing;
#[path = "native_graph_input_guards/path_value.rs"]
mod path_value;
#[path = "native_graph_input_guards/path_value_admission.rs"]
mod path_value_admission;
#[path = "native_graph_input_guards/path_value_binding.rs"]
mod path_value_binding;
#[path = "native_graph_input_guards/path_value_evidence.rs"]
mod path_value_evidence;
#[path = "native_graph_input_guards/path_value_targets.rs"]
mod path_value_targets;
#[path = "native_graph_input_guards/program.rs"]
mod program;
#[path = "native_graph_input_guards/retiring_actions.rs"]
mod retiring_actions;
#[path = "native_graph_input_guards/retiring_admission.rs"]
mod retiring_admission;
#[path = "native_graph_input_guards/retiring_evidence.rs"]
mod retiring_evidence;
#[path = "native_graph_input_guards/reuse_actions.rs"]
mod reuse_actions;
#[path = "native_graph_input_guards/reuse_admission.rs"]
mod reuse_admission;
#[path = "native_graph_input_guards/reuse_evidence.rs"]
mod reuse_evidence;
#[path = "native_graph_input_guards/reuse_note_codec.rs"]
mod reuse_note_codec;
#[path = "native_graph_input_guards/revision_query.rs"]
mod revision_query;
#[path = "native_graph_input_guards/row_query.rs"]
mod row_query;
#[path = "native_graph_input_guards/schema.rs"]
mod schema;
#[path = "native_graph_input_guards/selection_schema.rs"]
mod selection_schema;
#[path = "native_graph_input_guards/snapshot_query.rs"]
mod snapshot_query;
#[path = "native_graph_input_guards/world.rs"]
mod world;
#[path = "application_graph/document_retention_model/host/world_resources.rs"]
#[allow(
    dead_code,
    reason = "Reuse the ordinary bounded World policy, not its separate scale profiles."
)]
mod world_resources;
