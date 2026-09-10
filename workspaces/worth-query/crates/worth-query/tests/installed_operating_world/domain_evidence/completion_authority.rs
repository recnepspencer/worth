use worth_query::facade::installed::operation::WorthQueryAdmittedDirectOperation;
use worth_query::facade::{domain, foundation, runtime};

mod occurrence_oracle;

use occurrence_oracle::{expected_direct_occurrence, expected_workflow_occurrence};

use super::super::installed_operation_fixture::{
    evidence_graph_workflow_workspace, evidence_graph_workspace, evidence_workflow_intent,
    evidence_workflow_workspace, evidence_workspace, execution_resource_request, EvidenceFamily,
    EvidenceRead, EvidenceScenario, GeometryDomain, ReadFamily, WorkflowRead,
};

#[test]
fn exact_direct_completions_bind_zero_graph_output_snapshot_and_resource_attempt() {
    let mut workspace = evidence_workspace(
        "domain-evidence-completion-twins",
        EvidenceScenario::Honest,
        domain::WorthQueryArtifactRedactionPosture::NotRequired,
    )
    .unwrap();
    let first_admitted = admit_exact_completion(&workspace);
    let second_admitted = admit_exact_completion(&workspace);
    let first = execute_exact_completion(first_admitted, &mut workspace, 0);
    let second = execute_exact_completion(second_admitted, &mut workspace, 0);

    assert_eq!(
        first.output, second.output,
        "the twin must hold output constant"
    );
    assert_eq!(first.operation, second.operation);
    assert_eq!(first.binding, second.binding);
    assert_eq!(first.basis, second.basis);
    assert_eq!(first.snapshot, second.snapshot);
    assert_ne!(first.provider_session, second.provider_session);
    assert_ne!(first.provider_attempt, second.provider_attempt);
    assert_ne!(first.execution_occurrence, second.execution_occurrence);
}

#[test]
fn exact_workflow_completions_bind_stage_run_snapshot_and_resource_attempt() {
    let (mut workspace, _) =
        evidence_workflow_workspace("domain-evidence-workflow-completion-twins").unwrap();
    let first = execute_workflow_completion(&mut workspace, 0);
    let second = execute_workflow_completion(&mut workspace, 0);

    assert_eq!(
        first.output, second.output,
        "the stage output must remain constant"
    );
    assert_eq!(first.operation, second.operation);
    assert_eq!(first.binding, second.binding);
    assert_eq!(first.basis, second.basis);
    assert_eq!(first.stage, second.stage);
    assert_eq!(first.snapshot, second.snapshot);
    assert_ne!(first.run, second.run);
    assert_ne!(first.provider_session, second.provider_session);
    assert_ne!(first.provider_attempt, second.provider_attempt);
    assert_ne!(first.execution_occurrence, second.execution_occurrence);
}

#[test]
fn exact_direct_completions_bind_ordered_nonempty_graph_receipts() {
    let mut workspace = evidence_graph_workspace("domain-evidence-completion-graph-twins").unwrap();
    let first_admitted = admit_exact_completion(&workspace);
    let second_admitted = admit_exact_completion(&workspace);
    let first = execute_exact_completion(first_admitted, &mut workspace, 1);
    let second = execute_exact_completion(second_admitted, &mut workspace, 1);

    assert_equal_direct_semantics(&first, &second);
    assert_eq!(first.graph_receipts.len(), 1);
    assert_eq!(second.graph_receipts.len(), 1);
    assert_ne!(first.graph_receipts, second.graph_receipts);
    assert_ne!(first.execution_occurrence, second.execution_occurrence);
}

#[test]
fn exact_workflow_completions_bind_ordered_nonempty_graph_receipts() {
    let (mut workspace, _) =
        evidence_graph_workflow_workspace("domain-evidence-workflow-graph-twins").unwrap();
    let first = execute_workflow_completion(&mut workspace, 1);
    let second = execute_workflow_completion(&mut workspace, 1);

    assert_equal_workflow_semantics(&first, &second);
    assert_eq!(first.graph_receipts.len(), 1);
    assert_eq!(second.graph_receipts.len(), 1);
    assert_ne!(first.graph_receipts, second.graph_receipts);
    assert_ne!(first.execution_occurrence, second.execution_occurrence);
}

struct ExactCompletionObservation {
    operation: String,
    binding: String,
    basis: String,
    snapshot: String,
    output: String,
    provider_session: String,
    provider_attempt: String,
    graph_receipts: Vec<String>,
    execution_occurrence: String,
}

struct ExactWorkflowCompletionObservation {
    operation: String,
    binding: String,
    basis: String,
    stage: String,
    snapshot: String,
    output: String,
    run: String,
    provider_session: String,
    provider_attempt: String,
    graph_receipts: Vec<String>,
    execution_occurrence: String,
}

fn execute_workflow_completion(
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

type AdmittedExactDirect = WorthQueryAdmittedDirectOperation<
    GeometryDomain,
    EvidenceRead,
    EvidenceFamily,
    foundation::ObservationLaneWitness,
>;

fn admit_exact_completion(workspace: &runtime::WorthQueryWorkspace) -> AdmittedExactDirect {
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

fn execute_exact_completion(
    admitted: AdmittedExactDirect,
    workspace: &mut runtime::WorthQueryWorkspace,
    expected_graph_receipts: usize,
) -> ExactCompletionObservation {
    let expected_session = admitted.provider_session_identity().to_owned();
    let expected_attempt = admitted.provider_session_attempt_identity().to_owned();
    let expected_snapshot = workspace
        .snapshot_identity()
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

fn assert_equal_direct_semantics(
    first: &ExactCompletionObservation,
    second: &ExactCompletionObservation,
) {
    assert_eq!(first.output, second.output);
    assert_eq!(first.operation, second.operation);
    assert_eq!(first.binding, second.binding);
    assert_eq!(first.basis, second.basis);
    assert_eq!(first.snapshot, second.snapshot);
}

fn assert_equal_workflow_semantics(
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
