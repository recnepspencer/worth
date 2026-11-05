//! Moved whole from `domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity`: the items below and every impl they own.

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryAcceptedOutputCheckpointPosture {
    Performed,
    StableReused,
}

/// Borrowed checkpoint slot identity; it carries no publication authority.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct CheckpointOutputSlot<'a> {
    producer: &'a str,
    scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
    partition: [u8; 32],
}

impl<'a> CheckpointOutputSlot<'a> {
    pub(in crate::domain_computation::primary_graph) fn new(
        producer: &'a str,
        scope: crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding,
        partition: [u8; 32],
    ) -> Self {
        Self {
            producer,
            scope,
            partition,
        }
    }
}
