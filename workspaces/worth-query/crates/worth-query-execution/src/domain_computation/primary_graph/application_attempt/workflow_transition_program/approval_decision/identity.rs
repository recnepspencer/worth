use sha2::{Digest, Sha256};

use super::*;

pub(super) fn derive<Schema, Operation, Input, Scope>(
    compiled: &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
    approval_node: worth_relational::facade::identity::EntityId,
    required: &RequiredWorkflowApproval,
    proposal: &super::super::super::PublishedWorkflowProposalRef,
    decision: WorkflowApprovalDecision,
    admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    transitions: &[super::super::super::workflow_instance_observation::ObservedWorkflowTransition],
) -> Result<
    crate::domain_computation::primary_graph::workflow::WorkflowApprovalMeaning,
    WorthQueryApplicationAttemptDenial,
> {
    let authority = admission
        .workflow_approval_authority()
        .ok_or_else(|| affinity("workflow approval lacks retained capability authorization"))?;
    let evidence_join = unique(
        compiled.approval_evidence_sources(approval_node),
        "evidence",
    )?;
    let mut evidence = Vec::new();
    let mut evidence_material = Vec::new();
    for node in compiled.required_assessments(evidence_join.entity()) {
        let observed =
            super::super::super::workflow_instance_observation::latest_assessment_evidence(
                transitions,
                node.entity(),
            )
            .ok_or_else(|| affinity("approval required evidence is absent"))?;
        if !observed.passing {
            return Err(affinity("approval required evidence is failing"));
        }
        evidence.push(observed.entity);
        evidence_material.push((
            node.path().to_owned(),
            observed.identity.clone(),
            observed.source_identity.clone(),
            observed.publication_identity.clone(),
            observed.output_content_identity.clone(),
        ));
    }
    if evidence.len() < 2 {
        return Err(affinity("approval evidence inventory is incomplete"));
    }
    evidence_material.sort();
    let mut digest = Sha256::new();
    digest.update(b"worth-query:workflow-approval:v1");
    for value in [
        required.node_path(),
        required.transition_identity(),
        required.capability(),
        required.capability_type(),
        required.operation(),
        required.installed_capability_identity(),
        required.target_operation(),
        proposal.identity(),
        proposal.operation(),
        proposal.input_type(),
        compiled.content_identity().to_string().as_str(),
        compiled.program_revision().to_string().as_str(),
    ] {
        append(&mut digest, value);
    }
    digest.update(required.occurrence().to_le_bytes());
    digest.update(proposal.input_identity());
    append_entity(&mut digest, required.instance());
    append_entity(&mut digest, authority.principal);
    append_entity(&mut digest, authority.grant);
    append_entity(&mut digest, admission.scope_entity_id());
    append(&mut digest, authority.timeline.canonical_name());
    append(&mut digest, &format!("{:?}", authority.action));
    append(&mut digest, &format!("{:?}", authority.purpose));
    append(&mut digest, &format!("{:?}", authority.expiry));
    digest.update([match decision {
        WorkflowApprovalDecision::Approve => 1,
        WorkflowApprovalDecision::Reject => 2,
    }]);
    for (path, identity, source, publication, output) in evidence_material {
        for value in [path, identity, source, publication, output] {
            append(&mut digest, &value);
        }
    }
    let identity: [u8; 32] = digest.finalize().into();
    Ok(
        crate::domain_computation::primary_graph::workflow::WorkflowApprovalMeaning {
            identity,
            identity_text: hex(identity),
            instance_identity: format!("{:?}", required.instance()),
            definition_content_identity: compiled.content_identity().to_string(),
            program_revision: compiled.program_revision().to_string(),
            proposal: proposal.entity_id(),
            evidence: evidence.into_boxed_slice(),
            approver: authority.principal,
            grant: authority.grant,
            authorization_decision: authority.decision_identity,
            action: format!("{:?}", authority.action),
            purpose: format!("{:?}", authority.purpose),
            timeline: authority.timeline.canonical_name(),
            sampled_value: format!("{:?}", authority.sampled_value),
            expiry: authority.expiry,
            scope: admission.scope_entity_id(),
            approval_operation: required.operation().to_owned(),
            target_operation: required.target_operation().to_owned(),
            installed_capability_identity: required.installed_capability_identity().to_owned(),
            decision,
        },
    )
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
