pub use worth_query::facade::installed::operation::WorthQueryAdmittedDirectOperation;
pub use worth_query::facade::{domain, foundation, runtime};

pub mod occurrence_oracle;

pub use occurrence_oracle::{expected_direct_occurrence, expected_workflow_occurrence};

pub use super::super::installed_operation_fixture::{
    evidence_graph_workflow_workspace, evidence_graph_workspace, evidence_workflow_intent,
    evidence_workflow_workspace, evidence_workspace, execution_resource_request, EvidenceFamily,
    EvidenceRead, EvidenceScenario, GeometryDomain, ReadFamily, WorkflowRead,
};

pub struct ExactCompletionObservation {
    pub operation: String,
    pub binding: String,
    pub basis: String,
    pub snapshot: String,
    pub output: String,
    pub provider_session: String,
    pub provider_attempt: String,
    pub graph_receipts: Vec<String>,
    pub execution_occurrence: String,
}

pub struct ExactWorkflowCompletionObservation {
    pub operation: String,
    pub binding: String,
    pub basis: String,
    pub stage: String,
    pub snapshot: String,
    pub output: String,
    pub run: String,
    pub provider_session: String,
    pub provider_attempt: String,
    pub graph_receipts: Vec<String>,
    pub execution_occurrence: String,
}

pub fn execute_workflow_completion(
    workspace: &mut runtime::WorthQueryWorkspace,
    expected_graph_receipts: usize,
) -> ExactWorkflowCompletionObservation {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let trace = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(execution_resource_request(), workspace)
        .unwrap()
        .reexecute(evidence_workflow_intent(), workspace)
        .unwrap();
    let receipt = trace
        .stage_receipts()
        .iter()
        .find(|receipt| receipt.stage_identity() == "start")
        .unwrap();
    assert_eq!(receipt.graph_receipts().len(), expected_graph_receipts);
    let graph_receipts = receipt
        .graph_receipts()
        .iter()
        .map(|receipt| receipt.evidence_identity().to_owned())
        .collect::<Vec<_>>();
    let evidence = receipt
        .domain_evidence()
        .expect("stage evidence is required");
    let binding = evidence.binding();
    assert_eq!(binding.operation_identity(), receipt.operation_identity());
    assert_eq!(binding.binding_identity(), receipt.binding_identity());
    assert_eq!(binding.basis_identity(), receipt.basis_identity());
    assert_eq!(binding.run_identity(), Some(receipt.run_identity()));
    assert_eq!(binding.stage_identity(), Some(receipt.stage_identity()));
    assert_eq!(
        binding.execution_snapshot_identity(),
        receipt
            .execution_snapshot()
            .evidence_identity()
            .terminal_projection_for_reporting()
    );
    assert_eq!(
        binding.execution_occurrence_identity(),
        expected_workflow_occurrence(binding, receipt.execution_resources(), &graph_receipts)
    );
    ExactWorkflowCompletionObservation {
        operation: binding.operation_identity().to_owned(),
        binding: binding.binding_identity().to_owned(),
        basis: binding.basis_identity().to_owned(),
        stage: binding.stage_identity().unwrap().to_owned(),
        snapshot: binding.execution_snapshot_identity().to_owned(),
        output: binding.output_occurrence_identity().to_owned(),
        run: receipt.run_identity().to_owned(),
        provider_session: receipt
            .execution_resources()
            .provider_session_identity()
            .to_owned(),
        provider_attempt: receipt
            .execution_resources()
            .provider_session_attempt_identity()
            .to_owned(),
        graph_receipts,
        execution_occurrence: binding.execution_occurrence_identity().to_owned(),
    }
}

pub type AdmittedExactDirect = WorthQueryAdmittedDirectOperation<
    GeometryDomain,
    EvidenceRead,
    EvidenceFamily,
    foundation::ObservationLaneWitness,
>;

pub fn admit_exact_completion(workspace: &runtime::WorthQueryWorkspace) -> AdmittedExactDirect {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(EvidenceFamily)
        .bind(&installed, EvidenceRead)
        .unwrap()
        .admit_execution_resources((), execution_resource_request(), workspace)
        .unwrap()
}

pub fn execute_exact_completion(
    admitted: AdmittedExactDirect,
    workspace: &mut runtime::WorthQueryWorkspace,
    expected_graph_receipts: usize,
) -> ExactCompletionObservation {
    let expected_session = admitted.provider_session_identity().to_owned();
    let expected_attempt = admitted.provider_session_attempt_identity().to_owned();
    let expected_snapshot = workspace
        .with_advancement(|phase| workspace.snapshot_identity(&phase))
        .expect("installed host call opens")
        .expect("phase belongs to workspace")
        .evidence_identity()
        .terminal_projection_for_reporting()
        .to_owned();
    let executed = admitted.execute(workspace).unwrap();
    assert_eq!(executed.graph_receipts().len(), expected_graph_receipts);
    let graph_receipts = executed
        .graph_receipts()
        .iter()
        .map(|receipt| receipt.evidence_identity().to_owned())
        .collect::<Vec<_>>();
    let expected_output = executed.completed_output_occurrence_identity();
    let receipt = executed.receipt();
    let evidence = receipt
        .domain_evidence()
        .expect("evidence contract is required");
    let binding = evidence.binding();
    assert_eq!(receipt.output_identity(), expected_output);
    assert_eq!(binding.execution_snapshot_identity(), expected_snapshot);
    assert_eq!(binding.output_occurrence_identity(), expected_output);
    assert_eq!(
        receipt.execution_resources().provider_session_identity(),
        expected_session
    );
    assert_eq!(
        receipt
            .execution_resources()
            .provider_session_attempt_identity(),
        expected_attempt
    );
    assert_eq!(
        binding.execution_occurrence_identity(),
        expected_direct_occurrence(binding, receipt.execution_resources(), &graph_receipts)
    );
    ExactCompletionObservation {
        operation: binding.operation_identity().to_owned(),
        binding: binding.binding_identity().to_owned(),
        basis: binding.basis_identity().to_owned(),
        snapshot: binding.execution_snapshot_identity().to_owned(),
        output: expected_output,
        provider_session: expected_session,
        provider_attempt: expected_attempt,
        graph_receipts,
        execution_occurrence: binding.execution_occurrence_identity().to_owned(),
    }
}

pub fn assert_equal_direct_semantics(
    first: &ExactCompletionObservation,
    second: &ExactCompletionObservation,
) {
    assert_eq!(first.output, second.output);
    assert_eq!(first.operation, second.operation);
    assert_eq!(first.binding, second.binding);
    assert_eq!(first.basis, second.basis);
    assert_eq!(first.snapshot, second.snapshot);
}

pub fn assert_equal_workflow_semantics(
    first: &ExactWorkflowCompletionObservation,
    second: &ExactWorkflowCompletionObservation,
) {
    assert_eq!(first.output, second.output);
    assert_eq!(first.operation, second.operation);
    assert_eq!(first.binding, second.binding);
    assert_eq!(first.basis, second.basis);
    assert_eq!(first.stage, second.stage);
    assert_eq!(first.snapshot, second.snapshot);
}
