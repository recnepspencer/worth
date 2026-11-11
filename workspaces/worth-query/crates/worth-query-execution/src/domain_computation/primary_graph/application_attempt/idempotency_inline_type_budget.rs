//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryApplicationIdempotencyBinding>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/idempotency.rs", "WorthQueryApplicationIdempotencyBinding", 128 * 1024);
        owner.field::<[u8; 32]>("key_identity");
        owner.field::<[u8; 32]>("intent_identity");
        owner.field::<Option<[u8; 32]>>("source_identity");
        owner.field::<Option<[u8; 32]>>("source_partition_identity");
        owner.field::<Option<[u8; 32]>>("producer_dependency_identity");
        owner
            .field::<Option<worth_relational::facade::mvcc::RequiredOutputProducingImplementation>>(
                "producing_implementation",
            );
        owner.field::<Option<[u8; 32]>>("operation_identity");
        owner.field::<Option<WorthQueryIdempotencyScopeIdentity>>("operation_scope_identity");
        owner.field::<Option<[u8; 32]>>("precondition_identity");
        owner.field::<Option<[u8; 32]>>("governed_input_identity");
        owner.field::<Option<[u8; 32]>>("governed_proposal_identity");
        owner.field::<Option<[u8; 32]>>("conditional_definition_identity");
        owner.field::<Option<[u8; 32]>>("workflow_definition_identity");
        owner.field::<Option<[u8; 32]>>("workflow_instance_identity");
        owner.field::<workflow_proposal_context::WorkflowProposalIdempotency>("workflow_proposal");
        owner.field::<Option<[u8; 32]>>("workflow_transition_identity");
        owner.field::<Option<[u8; 32]>>("workflow_support_identity");
        owner.field::<Option<[u8; 32]>>("workflow_client_key_identity");
        owner.field::<Option<[u8; 32]>>("workflow_approval_identity");
        owner.field::<Option<[u8; 32]>>("mutation_binding_identity");
        owner.field::<Option<[u8; 32]>>("correction_identity");
        owner.field::<Option<[u8; 32]>>("transport_intent_identity");
    }
}
