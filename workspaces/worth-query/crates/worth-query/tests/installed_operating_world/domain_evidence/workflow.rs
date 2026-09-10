use worth_proof::TransitionOutcome;
use worth_query::facade::{certification, domain, foundation, runtime};

use super::super::installed_operation_fixture::{
    evidence_workflow_intent, evidence_workflow_workspace, EvidenceWorkflowMode, GeometryDomain,
    ReadFamily, WorkflowRead,
};

#[test]
fn workflow_run_ledger_denies_a_locally_valid_counter_regression_atomically() {
    let (mut workspace, probe) =
        evidence_workflow_workspace("domain-evidence-ledger-regression").unwrap();
    probe.set(EvidenceWorkflowMode::LedgerRegression);

    let denial = match bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(evidence_workflow_intent(), &mut workspace)
    {
        TransitionOutcome::Denied(domain::WorthQueryWorkflowReexecutionStop::Advance(denial)) => {
            denial
        }
        _ => panic!("the operation-scoped counter regression did not deny workflow advancement"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryWorkflowAdvanceDenialKind::DomainEvidence(
            domain::WorthQueryDomainEvidenceAdmissionDenialKind::LedgerRegression
        )
    );
    assert_eq!(denial.completed_stage_receipts().len(), 2);
    let start = stage_evidence(denial.completed_stage_receipts(), "start");
    assert_eq!(counter(start, "candidate-comparisons").initial(), 0);
    assert_eq!(counter(start, "candidate-comparisons").observed(), 6);
    assert!(denial
        .completed_stage_receipts()
        .iter()
        .all(|receipt| receipt.stage_identity() != "left"));
}

#[test]
fn claimed_output_mismatch_denies_before_a_stage_receipt_exists() {
    let (mut workspace, probe) =
        evidence_workflow_workspace("domain-evidence-workflow-output-mismatch").unwrap();
    probe.set(EvidenceWorkflowMode::OutputOccurrenceMismatch);

    let denial = match bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(evidence_workflow_intent(), &mut workspace)
    {
        TransitionOutcome::Denied(domain::WorthQueryWorkflowReexecutionStop::Advance(denial)) => {
            denial
        }
        TransitionOutcome::Success(_) => panic!("mismatched output produced a stage receipt"),
        _ => panic!("mismatched output did not deny exact stage completion"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryWorkflowAdvanceDenialKind::DomainEvidence(
            domain::WorthQueryDomainEvidenceAdmissionDenialKind::TransformationSummaryMismatch,
        )
    );
    assert!(denial.completed_stage_receipts().is_empty());
    assert!(denial.graph_receipts().is_empty());
}

#[test]
fn replay_compares_mandatory_core_and_ignores_policy_omitted_sidecars() {
    let (mut workspace, probe) =
        evidence_workflow_workspace("domain-evidence-replay-sidecars").unwrap();
    let original = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(evidence_workflow_intent(), &mut workspace)
        .unwrap();
    probe.set(EvidenceWorkflowMode::OmitSidecars);
    let candidate = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(evidence_workflow_intent(), &mut workspace)
        .unwrap();

    for stage in ["start", "left"] {
        let original_evidence = stage_evidence(original.stage_receipts(), stage);
        let candidate_evidence = stage_evidence(candidate.stage_receipts(), stage);
        assert_eq!(original_evidence.core(), candidate_evidence.core());
        assert!(original_evidence.counter_sidecar().records().is_some());
        assert!(original_evidence.decision_sidecar().records().is_some());
        assert!(original_evidence.candidate_sidecar().records().is_some());
        assert!(original_evidence
            .transformation_sidecar()
            .records()
            .is_some());
        assert!(matches!(
            candidate_evidence.counter_sidecar(),
            domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
        ));
        assert!(matches!(
            candidate_evidence.decision_sidecar(),
            domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
        ));
        assert!(matches!(
            candidate_evidence.candidate_sidecar(),
            domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
        ));
        assert!(matches!(
            candidate_evidence.transformation_sidecar(),
            domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
        ));
    }
    assert_eq!(
        domain::compare_exact_workflow_traces(
            &original.semantics(),
            &candidate.semantics(),
            Default::default(),
        ),
        domain::WorthQueryReplayComparison::Equivalent
    );

    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind(&workspace),
        evidence_workflow_intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    assert_eq!(replay.counters().semantic_stage_comparisons, 4);
}

#[test]
fn certification_replay_cannot_waive_exact_mandatory_core_drift() {
    let (mut workspace, probe) =
        evidence_workflow_workspace("domain-evidence-replay-core-drift").unwrap();
    let original = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(evidence_workflow_intent(), &mut workspace)
        .unwrap();
    probe.set(EvidenceWorkflowMode::ReplayCoreDrift);

    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind(&workspace),
        evidence_workflow_intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Diverged(
            domain::WorthQueryReplayDivergence::DomainEvidence {
                stage: "left".into()
            }
        )
    );
    assert_eq!(replay.counters().semantic_stage_comparisons, 1);
}

fn bind(
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

fn stage_evidence<'a>(
    receipts: &'a [domain::WorthQueryWorkflowStageReceipt],
    stage: &str,
) -> &'a domain::WorthQueryAdmittedDomainEvidence {
    receipts
        .iter()
        .find(|receipt| receipt.stage_identity() == stage)
        .and_then(domain::WorthQueryWorkflowStageReceipt::domain_evidence)
        .unwrap_or_else(|| panic!("missing admitted evidence for workflow stage {stage}"))
}

fn counter<'a>(
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
