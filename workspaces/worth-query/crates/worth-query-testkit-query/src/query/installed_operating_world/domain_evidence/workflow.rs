pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{certification, domain, foundation, runtime};

pub use super::super::installed_operation_fixture::{
    evidence_workflow_intent, evidence_workflow_workspace, EvidenceWorkflowMode, GeometryDomain,
    ReadFamily, WorkflowRead,
};

pub fn bind(
    workspace: &runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
}

pub fn stage_evidence<'a>(
    receipts: &'a [domain::WorthQueryWorkflowStageReceipt],
    stage: &str,
) -> &'a domain::WorthQueryAdmittedDomainEvidence {
    receipts
        .iter()
        .find(|receipt| receipt.stage_identity() == stage)
        .and_then(domain::WorthQueryWorkflowStageReceipt::domain_evidence)
        .unwrap_or_else(|| panic!("missing admitted evidence for workflow stage {stage}"))
}

pub fn counter<'a>(
    evidence: &'a domain::WorthQueryAdmittedDomainEvidence,
    name: &str,
) -> &'a domain::WorthQueryAdmittedStructuralCounter {
    evidence
        .core()
        .counters()
        .iter()
        .find(|counter| counter.schema().name().as_str() == name)
        .unwrap_or_else(|| panic!("missing admitted workflow counter {name}"))
}
