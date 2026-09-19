use worth_foundational::FoundationalBoundaryEvidenceContinuityAttachmentScope;
use worth_proof::TransitionOutcome;
use worth_query::facade::{certification, domain, foundation};

use super::installed_operation_fixture::{
    conditional_workflow_workspace, missing_replay_comparator_workspace, workflow_workspace,
    GeometryDomain, ReadFamily, WorkflowRead,
};

#[test]
fn replay_comparator_must_be_installed_before_any_replay_can_execute() {
    let denial = match missing_replay_comparator_workspace("missing-replay-comparator") {
        Ok(_) => panic!("replay contract without a comparator object must not install"),
        Err(denial) => denial,
    };
    assert!(denial
        .message()
        .contains("workflow replay comparator disagrees with installed semantics"));
}

#[test]
fn ordinary_reexecution_uses_installed_intent_and_mints_a_distinct_run() {
    let mut workspace = workflow_workspace("ordinary-reexecution").unwrap();
    let original_bound = bind(&workspace);
    let original = original_bound
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();
    let replay_bound = bind(&workspace);
    let reexecuted = replay_bound
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();

    assert_ne!(original.identity(), reexecuted.identity());
    assert_ne!(
        original.stage_receipts()[0].run_identity(),
        reexecuted.stage_receipts()[0].run_identity()
    );
    assert_eq!(
        domain::compare_exact_workflow_traces(
            &original.semantics(),
            &reexecuted.semantics(),
            Default::default(),
        ),
        domain::WorthQueryReplayComparison::Equivalent
    );
    assert_eq!(reexecuted.counters().stage_executor_contacts, 4);
    assert!(reexecuted.publish().is_success());
}

#[test]
fn certification_replay_is_trace_bound_and_denies_foreign_basis_before_execution() {
    let mut workspace = workflow_workspace("certification-replay").unwrap();
    let original = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();
    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind(&workspace),
        intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    assert_ne!(
        replay.original_trace_identity(),
        replay.replay_trace_identity()
    );
    assert_eq!(replay.counters().semantic_stage_comparisons, 4);
    assert_eq!(replay.counters().original_stage_index_entries, 4);
    assert_eq!(replay.counters().unrelated_trace_scans, 0);
    assert_eq!(
        replay.original_execution_counters(),
        replay.replay_execution_counters()
    );
    let foundational = replay.foundational_attachment();
    assert_ne!(
        foundational.original_trace_evidence_identity(),
        foundational.replay_trace_evidence_identity()
    );
    assert_eq!(
        foundational.materialized().continuity_scope(),
        Some(FoundationalBoundaryEvidenceContinuityAttachmentScope::ObjectLevel)
    );
    assert_eq!(
        foundational.admit_current_basis().payload().target(),
        foundational.materialized().target()
    );

    let foreign_workspace = workflow_workspace("foreign-certification-replay").unwrap();
    let foreign_bound = bind(&foreign_workspace);
    let denial = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        foreign_bound,
        intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    );
    assert!(matches!(
        denial,
        TransitionOutcome::Denied(certification::WorthQueryCertificationReplayStop::Admission(
            certification::WorthQueryCertificationReplayAdmissionDenial::ForeignRuntime
        ))
    ));
}

#[test]
fn certification_replay_localizes_realized_conditional_path_drift() {
    let dependency = super::conditional_node_contract::dependency(
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let node = domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        "publish-when-changed",
        domain::WorthQueryConditionalNodeRole::WorkflowStage,
    )
    .dependencies([dependency.clone()])
    .outputs([
        domain::WorthQueryConditionalNodeOutput::WorkflowStageOutput {
            contract: domain::WorthQueryWorkflowValueContract::Projection,
        },
    ])
    .required_context([domain::WorthQueryConditionalNodeContext::WorkflowRun])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([dependency]).unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
    )
    .comparison(
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::NotReusable,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        domain::WorthQueryArtifactPosture::Ephemeral,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::IsWorkflowStageOutput)
    .finish()
    .unwrap();
    let mut workspace = conditional_workflow_workspace("conditional-replay", node).unwrap();
    let original = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();
    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind(&workspace),
        intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    );
    assert!(matches!(
        replay,
        TransitionOutcome::Deferred(
            certification::WorthQueryCertificationReplayStop::SemanticDivergence(
                domain::WorthQueryReplayDivergence::ConditionalPath { stage }
            )
        ) if stage == "publish"
    ));
}

#[test]
fn historical_replay_resolves_owner_evidence_for_the_exact_basis_pair() {
    let mut workspace = workflow_workspace("historical-certification-replay").unwrap();
    let historical_context = worth_query::facade::history::at(&workspace);
    let original = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();
    let historical_bound = bind(&workspace);
    let admission = certification::admit_installed_historical_replay_basis(
        certification::issue_query_certification_replay_capability(),
        &original,
        &historical_bound,
        &historical_context,
        certification::WorthQueryInstalledHistoricalReplayPath::RetainedSnapshot,
    )
    .unwrap();
    assert!(!admission.original_basis_identity().is_empty());
    assert_eq!(
        admission.historical_snapshot_identity(),
        historical_context.snapshot_identity()
    );

    let admission_correspondence = admission.correspondence();
    let replay = certification::replay_installed_workflow_historical(
        admission,
        &original,
        historical_bound,
        intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    assert!(matches!(
        replay.basis_relationship(),
        certification::WorthQueryReplayBasisRelationship::AdmittedHistoricalBasis {
            correspondence,
        } if correspondence == admission_correspondence
    ));
}

#[test]
fn historical_replay_refuses_to_simulate_an_unowned_reconstruction_path() {
    let mut workspace = workflow_workspace("historical-reconstruction-denial").unwrap();
    let historical_context = worth_query::facade::history::at(&workspace);
    let original = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();
    let bound = bind(&workspace);

    let denied = certification::admit_installed_historical_replay_basis(
        certification::issue_query_certification_replay_capability(),
        &original,
        &bound,
        &historical_context,
        certification::WorthQueryInstalledHistoricalReplayPath::DeltaReplay {
            max_events: 1,
            actual_events: 1,
        },
    );
    assert!(matches!(
        denied,
        TransitionOutcome::Denied(
            certification::WorthQueryHistoricalReplayAdmissionDenial::HistoricalExecutionSubstrateUnavailable
        )
    ));
}

#[test]
fn historical_replay_denies_when_the_retained_execution_substrate_has_drifted() {
    let mut workspace = workflow_workspace("historical-replay-drift").unwrap();
    let historical_context = worth_query::facade::history::at(&workspace);
    let original = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();
    let replay_bound = bind(&workspace);
    let admission = certification::admit_installed_historical_replay_basis(
        certification::issue_query_certification_replay_capability(),
        &original,
        &replay_bound,
        &historical_context,
        certification::WorthQueryInstalledHistoricalReplayPath::RetainedSnapshot,
    )
    .unwrap();
    workspace
        .insert("Vertex", |builder| {
            builder.aspect("identity.id", "historical-replay-drift")
        })
        .unwrap();

    let denied = certification::replay_installed_workflow_historical(
        admission,
        &original,
        replay_bound,
        intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    );
    assert!(matches!(
        denied,
        TransitionOutcome::Denied(certification::WorthQueryCertificationReplayStop::Admission(
            certification::WorthQueryCertificationReplayAdmissionDenial::HistoricalExecutionBasisDrift
        ))
    ));
}

#[test]
fn retry_requires_installed_idempotence_and_never_reuses_attempt_identity() {
    let mut workspace = workflow_workspace("idempotent-stage-retry").unwrap();
    let run = bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap();
    let first = run
        .prepare_stage_attempt(
            "left",
            domain::WorthQueryWorkflowIntentValue::Text("fail-dependency".into()),
        )
        .unwrap();
    let first_identity = first.identity().to_owned();
    let failure = match first.execute(&mut workspace) {
        domain::WorthQueryWorkflowStageAttemptOutcome::Retryable(failure) => failure,
        _ => panic!("effect-free declared executor failure was not retryable"),
    };
    assert_eq!(failure.failed_attempt_identity(), first_identity);
    assert!(failure.denial().executed_effects().is_empty());
    let second = failure.retry();
    assert_ne!(second.identity(), first_identity);
    assert!(matches!(
        second.execute(&mut workspace),
        domain::WorthQueryWorkflowStageAttemptOutcome::Retryable(_)
    ));
}

pub(crate) fn intent() -> domain::WorthQueryNormalizedWorkflowIntent {
    use domain::{WorthQueryWorkflowIntentStage as Stage, WorthQueryWorkflowIntentValue as Value};
    domain::WorthQueryNormalizedWorkflowIntent::new(vec![
        Stage::new("start", Value::NotRequired),
        Stage::new("right", Value::Text("start".into())),
        Stage::new("left", Value::Text("start".into())),
        Stage::new("publish", Value::Text("join".into())),
    ])
    .unwrap()
}

fn bind(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
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
