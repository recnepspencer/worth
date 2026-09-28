use sha2::{Digest, Sha256};
use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventIntent;

use crate::domain_computation::primary_graph::{
    PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef, RequiredWorkflowApproval,
    WorkflowApprovalDecision,
};

/// Stable signing meaning survives navigation to another occurrence, while a
/// different proposal, definition, operation, decision or covered source does not.
pub(crate) fn workflow_approval_authentication_intent(
    instance: &PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    decision: WorkflowApprovalDecision,
) -> WorthQueryAuthenticationEventIntent {
    let mut signing = Sha256::new();
    signing.update(b"worth-query:workflow-approval-authentication:v1");
    append(&mut signing, &format!("{:?}", instance.branch()));
    append_entity(&mut signing, instance.entity_id());
    append_entity(&mut signing, instance.definition_entity_id());
    append(
        &mut signing,
        &instance.definition_content_identity().to_string(),
    );
    append(&mut signing, &instance.program_revision().to_string());
    for value in [
        required.node_path(),
        required.capability(),
        required.capability_type(),
        required.operation(),
        required.installed_capability_identity(),
        required.target_operation(),
        proposal.identity(),
        proposal.operation(),
        proposal.input_type(),
    ] {
        append(&mut signing, value);
    }
    signing.update(proposal.input_identity());
    signing.update([match decision {
        WorkflowApprovalDecision::Approve => 1,
        WorkflowApprovalDecision::Reject => 2,
    }]);

    let mut coverage = Sha256::new();
    coverage.update(b"worth-query:workflow-approval-coverage:v1");
    append_entity(&mut coverage, instance.entity_id());
    append_entity(&mut coverage, proposal.entity_id());
    append_entity(&mut coverage, proposal.instance_entity_id());
    append_entity(&mut coverage, proposal.definition_entity_id());
    append(&mut coverage, proposal.identity());
    coverage.update(proposal.input_identity());
    if let Some(source) = proposal.source_identity() {
        coverage.update([1]);
        coverage.update(source);
    } else {
        coverage.update([0]);
    }
    WorthQueryAuthenticationEventIntent::new(
        "workflow-approval-signature",
        CanonicalDigestId::new(signing.finalize().into()),
        CanonicalDigestId::new(coverage.finalize().into()),
    )
    .expect("the fixed Query approval purpose is valid")
}

fn append(digest: &mut Sha256, value: &str) {
    digest.update((value.len() as u64).to_le_bytes());
    digest.update(value.as_bytes());
}

fn append_entity(digest: &mut Sha256, entity: worth_relational::facade::identity::EntityId) {
    digest.update(entity.partition_value_u64().to_le_bytes());
    digest.update(entity.local_slot_value().to_le_bytes());
    digest.update(u64::from(entity.generation_value()).to_le_bytes());
}
