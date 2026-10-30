pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{domain, runtime};

pub use super::super::installed_operation_fixture::{
    evidence_workspace, evidence_workspace_with_governance, EvidenceFamily, EvidenceGovernance,
    EvidenceRead, EvidenceScenario, GeometryDomain,
};

pub fn admitted_receipt(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    scenario: EvidenceScenario,
    redaction: domain::WorthQueryArtifactRedactionPosture,
) -> OwnedExecutionReceipt {
    execute_receipt(evidence_workspace(request_execution, name, scenario, redaction).unwrap())
}

pub fn admitted_receipt_with_governance(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    scenario: EvidenceScenario,
    governance: EvidenceGovernance,
) -> OwnedExecutionReceipt {
    execute_receipt(
        evidence_workspace_with_governance(request_execution, name, scenario, governance).unwrap(),
    )
}

pub fn execute_receipt(mut workspace: runtime::WorthQueryWorkspace) -> OwnedExecutionReceipt {
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
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();
    OwnedExecutionReceipt::new(executed, |executed| executed.receipt())
}

pub struct OwnedExecutionReceipt {
    pub owner: Box<dyn ExecutionReceiptOwner>,
}

impl OwnedExecutionReceipt {
    pub fn new<T: 'static>(
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

pub trait ExecutionReceiptOwner {
    fn receipt(&self) -> &domain::WorthQueryBoundExecutionReceipt;
}

pub struct TypedExecutionReceiptOwner<T> {
    pub owner: T,
    pub receipt: fn(&T) -> &domain::WorthQueryBoundExecutionReceipt,
}

impl<T> ExecutionReceiptOwner for TypedExecutionReceiptOwner<T> {
    fn receipt(&self) -> &domain::WorthQueryBoundExecutionReceipt {
        (self.receipt)(&self.owner)
    }
}

pub fn denied_execution(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    scenario: EvidenceScenario,
) -> domain::WorthQueryBoundExecutionDenial {
    let mut workspace = evidence_workspace(
        request_execution,
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
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("dishonest domain evidence did not produce an admission denial"),
    }
}

pub fn settled_honest_execution(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    EvidenceRead,
    EvidenceFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    let mut workspace = evidence_workspace(
        request_execution,
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
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
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

pub fn evidence(
    receipt: &domain::WorthQueryBoundExecutionReceipt,
) -> &domain::WorthQueryAdmittedDomainEvidence {
    receipt
        .domain_evidence()
        .expect("the installed contract requires admitted domain evidence")
}

pub fn inspection(
    evidence: &domain::WorthQueryAdmittedDomainEvidence,
    policy: runtime::CausalInspectionRedactionPolicy,
) -> runtime::WorthQueryDomainEvidenceInspectionCopy {
    runtime::WorthQueryDomainEvidenceInspectionCopy::derive(evidence, policy)
}
