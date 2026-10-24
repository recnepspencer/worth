//! Original performed output identities, not present-day selection or command authority.
mod context;
mod origin;
mod publication;
pub(in crate::domain_computation::primary_graph) use context::decode_transition_output_context;
pub(in crate::domain_computation::primary_graph) use origin::WorkflowOperationOrigin;
pub(in crate::domain_computation::primary_graph) use publication::append_operation_outputs;
