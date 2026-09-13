use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, runtime};

use super::super::installed_operation_fixture::{
    evidence_workspace, evidence_workspace_with_governance, EvidenceFamily, EvidenceGovernance,
    EvidenceRead, EvidenceScenario, GeometryDomain,
};

pub(super) fn admitted_receipt(
    name: &str,
    scenario: EvidenceScenario,
    redaction: domain::WorthQueryArtifactRedactionPosture,
) -> OwnedExecutionReceipt {
    execute_receipt(evidence_workspace(name, scenario, redaction).unwrap())
}

pub(super) fn admitted_receipt_with_governance(
    name: &str,
    scenario: EvidenceScenario,
    governance: EvidenceGovernance,
) -> OwnedExecutionReceipt {
    execute_receipt(evidence_workspace_with_governance(name, scenario, governance).unwrap())
}

fn execute_receipt(mut workspace: runtime::WorthQueryWorkspace) -> OwnedExecutionReceipt {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(EvidenceFamily)
        .bind(&installed, EvidenceRead)
        .unwrap();
    let executed = bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();
    OwnedExecutionReceipt::new(executed, |executed| executed.receipt())
}

pub(super) struct OwnedExecutionReceipt {
    owner: Box<dyn ExecutionReceiptOwner>,
}

impl OwnedExecutionReceipt {
    fn new<T: 'static>(
        owner: T,
        receipt: fn(&T) -> &domain::WorthQueryBoundExecutionReceipt,
    ) -> Self {
        Self {
            owner: Box::new(TypedExecutionReceiptOwner { owner, receipt }),
        }
    }
}

impl std::ops::Deref for OwnedExecutionReceipt {
    type Target = domain::WorthQueryBoundExecutionReceipt;

    fn deref(&self) -> &Self::Target {
        self.owner.receipt()
    }
}

trait ExecutionReceiptOwner {
    fn receipt(&self) -> &domain::WorthQueryBoundExecutionReceipt;
}

struct TypedExecutionReceiptOwner<T> {
    owner: T,
    receipt: fn(&T) -> &domain::WorthQueryBoundExecutionReceipt,
}

impl<T> ExecutionReceiptOwner for TypedExecutionReceiptOwner<T> {
    fn receipt(&self) -> &domain::WorthQueryBoundExecutionReceipt {
        (self.receipt)(&self.owner)
    }
}

pub(super) fn denied_execution(
    name: &str,
    scenario: EvidenceScenario,
) -> domain::WorthQueryBoundExecutionDenial {
    let mut workspace = evidence_workspace(
        name,
        scenario,
        domain::WorthQueryArtifactRedactionPosture::NotRequired,
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(EvidenceFamily)
        .bind(&installed, EvidenceRead)
        .unwrap();
    match bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("dishonest domain evidence did not produce an admission denial"),
    }
}

pub(super) fn settled_honest_execution(
    name: &str,
) -> domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    EvidenceRead,
    EvidenceFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    let mut workspace = evidence_workspace(
        name,
        EvidenceScenario::Honest,
        domain::WorthQueryArtifactRedactionPosture::NotRequired,
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(EvidenceFamily)
        .bind(&installed, EvidenceRead)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(
            consumer,
            worth_query::facade::read::project_facts().entity_identities(),
        )
        .unwrap()
        .settle()
        .unwrap()
}

pub(super) fn evidence(
    receipt: &domain::WorthQueryBoundExecutionReceipt,
) -> &domain::WorthQueryAdmittedDomainEvidence {
    receipt
        .domain_evidence()
        .expect("the installed contract requires admitted domain evidence")
}

pub(super) fn inspection(
    evidence: &domain::WorthQueryAdmittedDomainEvidence,
    policy: runtime::CausalInspectionRedactionPolicy,
) -> runtime::WorthQueryDomainEvidenceInspectionCopy {
    runtime::WorthQueryDomainEvidenceInspectionCopy::derive(evidence, policy)
}
