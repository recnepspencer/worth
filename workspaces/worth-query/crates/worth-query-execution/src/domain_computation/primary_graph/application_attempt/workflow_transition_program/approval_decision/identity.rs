use sha2::{Digest, Sha256};
use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventIntent;

use super::inputs::ObservedApprovalEvidence;
use super::*;

pub(super) fn authentication_intent(
    instance: &super::super::super::PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &super::super::super::PublishedWorkflowProposalRef,
    decision: WorkflowApprovalDecision,
    evidence: &[ObservedApprovalEvidence],
    currentness: &[super::super::super::WorthQueryApplicationObservedFact],
) -> WorthQueryAuthenticationEventIntent {
    let basis = workflow_approval_authentication_intent(instance, required, proposal, decision);
    let mut coverage = Sha256::new();
    coverage.update(b"worth-query:approval-observed-coverage:v1");
    coverage.update(basis.subject_coverage().bytes());
    let mut entries = evidence
        .iter()
        .map(|item| {
            let observed = &item.evidence;
            format!(
                "{}|{:?}|{}|{}|{}|{}|{}|{}|{}",
                item.node_path,
                observed.subject,
                observed.coverage_identity,
                observed.identity,
                observed.proposal_identity,
                observed.source_identity,
                observed.publication_identity,
                observed.output_content_identity,
                observed.passing
            )
        })
        .collect::<Vec<_>>();
    entries.sort();
    for entry in entries {
        append(&mut coverage, &entry);
    }
    let mut dependencies = currentness
        .iter()
        .map(|fact| format!("{fact:?}"))
        .collect::<Vec<_>>();
    dependencies.sort();
    for dependency in dependencies {
        append(&mut coverage, &dependency);
    }
    let coverage: [u8; 32] = coverage.finalize().into();
    let mut signing = Sha256::new();
    signing.update(b"worth-query:approval-observed-signing-intent:v1");
    signing.update(basis.signing_intent().bytes());
    signing.update(coverage);
    WorthQueryAuthenticationEventIntent::new(
        basis.purpose(),
        CanonicalDigestId::new(signing.finalize().into()),
        CanonicalDigestId::new(coverage),
    )
    .expect("the Query-owned approval purpose is valid")
}

pub(super) fn derive<Schema, Operation, Input, Scope>(
    compiled: &crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition,
    required: &RequiredWorkflowApproval,
    proposal: &super::super::super::PublishedWorkflowProposalRef,
    decision: WorkflowApprovalDecision,
    admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    evidence_observations: &[ObservedApprovalEvidence],
) -> Result<
    crate::domain_computation::primary_graph::workflow::WorkflowApprovalMeaning,
    WorthQueryApplicationAttemptDenial,
> {
    let authority = admission
        .workflow_approval_authority()
        .ok_or_else(|| affinity("workflow approval lacks retained capability authorization"))?;
    let authorization_request = admission
        .workflow_approval_request_snapshot()
        .map_err(snapshot_denial)?
        .ok_or_else(|| affinity("workflow approval lacks retained capability request"))?;
    let authorization_principal = admission
        .workflow_approval_principal_snapshot()
        .map_err(snapshot_denial)?
        .ok_or_else(|| affinity("workflow approval lacks retained principal evidence"))?;
    let authorization_lineage = admission
        .workflow_approval_lineage_snapshot()
        .map_err(snapshot_denial)?
        .ok_or_else(|| affinity("workflow approval lacks retained capability lineage"))?;
    let authorization_support = admission
        .workflow_approval_support_snapshot()
        .map_err(snapshot_denial)?
        .ok_or_else(|| affinity("workflow approval lacks retained capability support state"))?;
    let authorization_dependencies = admission
        .workflow_approval_dependencies_snapshot()
        .map_err(snapshot_denial)?
        .ok_or_else(|| {
            affinity("workflow approval lacks retained native authority dependencies")
        })?;
    let mut evidence = Vec::new();
    let mut evidence_material = Vec::new();
    for observation in evidence_observations {
        let observed = &observation.evidence;
        if !observed.passing {
            return Err(affinity("approval required evidence is failing"));
        }
        evidence.push(observed.entity);
        evidence_material.push((
            observation.node_path.clone(),
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
    append(&mut digest, &authorization_request);
    append(&mut digest, &authorization_principal);
    append(&mut digest, &authority.capability_authority_identity);
    append(&mut digest, &authorization_lineage);
    append(&mut digest, &authorization_support);
    append(&mut digest, &authorization_dependencies);
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
            authorization_request,
            authorization_principal,
            capability_authority_identity: authority.capability_authority_identity,
            authorization_lineage,
            authorization_support,
            authorization_dependencies,
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

fn snapshot_denial(
    encoding: crate::domain_computation::authorization::WorkflowApprovalRequestEncodingDenial,
) -> WorthQueryApplicationAttemptDenial {
    use crate::domain_computation::authorization::WorkflowApprovalRequestEncodingDenial;
    let (kind, subject) = match encoding {
        WorkflowApprovalRequestEncodingDenial::TooLarge => (
            WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
            "workflow approval authorization snapshot exceeds its durable bound",
        ),
        WorkflowApprovalRequestEncodingDenial::NonPortableValue => (
            WorthQueryApplicationAttemptDenialKind::InvalidAuthoritativeValue,
            "workflow approval authorization snapshot contains a nonportable value",
        ),
        WorkflowApprovalRequestEncodingDenial::InvalidRequest => (
            WorthQueryApplicationAttemptDenialKind::InvalidAuthoritativeValue,
            "workflow approval authorization snapshot is inconsistent",
        ),
    };
    WorthQueryApplicationAttemptDenial::new(kind, subject)
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
