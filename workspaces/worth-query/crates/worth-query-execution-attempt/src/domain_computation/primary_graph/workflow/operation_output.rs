//! Original performed output identities, not present-day selection or command authority.
pub mod context;
pub mod origin;
pub mod publication;
pub use context::decode_transition_output_context;
pub use origin::WorkflowOperationOrigin;
pub use publication::append_operation_outputs;
